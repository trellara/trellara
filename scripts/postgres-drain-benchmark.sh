#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ROOT="${RUN_ROOT:-$ROOT/target/perf-drain-smalltx-$(date +%Y%m%d%H%M%S)}"
PGDATA="${PGDATA:-$RUN_ROOT/pgdata}"
PORT="${PORT:-56432}"
DB="${DB:-trellara_perf}"
ROLE="${ROLE:-trellara_perf}"
TARGET_GIB="${TARGET_GIB:-20}"
TARGET_MIB="${TARGET_MIB:-}"
TARGET_BYTES="${TARGET_BYTES:-}"
TX_BYTES="${TX_BYTES:-10240}"
TABLE_COUNT=5
PAYLOAD_POOL_SIZE="${PAYLOAD_POOL_SIZE:-4096}"
DRAIN_MODE="${DRAIN_MODE:-pg_recvlogical}"
DRAIN_TIMEOUT_SECONDS="${DRAIN_TIMEOUT_SECONDS:-7200}"
KEEP_POSTGRES="${KEEP_POSTGRES:-1}"
MAX_WAL_SIZE="${MAX_WAL_SIZE:-4GB}"
CHECKPOINT_AFTER_DRAIN="${CHECKPOINT_AFTER_DRAIN:-1}"
SCHEMA_CHANGE="${SCHEMA_CHANGE:-0}"
SCHEMA_CHANGE_AFTER_PERCENT="${SCHEMA_CHANGE_AFTER_PERCENT:-50}"
SCHEMA_CHANGE_AFTER_TX="${SCHEMA_CHANGE_AFTER_TX:-}"
SCHEMA_MARKER="${SCHEMA_MARKER:-schema-v2}"
TRELLARA_BIN="${TRELLARA_BIN:-$ROOT/target/debug/trellara}"

if [[ -x /opt/homebrew/opt/postgresql@17/bin/pg_ctl ]]; then
  PG_BIN_DEFAULT="/opt/homebrew/opt/postgresql@17/bin"
else
  PG_BIN_DEFAULT=""
fi

PG_CTL="${PG_CTL:-${PG_BIN_DEFAULT:+$PG_BIN_DEFAULT/pg_ctl}}"
INITDB="${INITDB:-${PG_BIN_DEFAULT:+$PG_BIN_DEFAULT/initdb}}"
PSQL="${PSQL:-${PG_BIN_DEFAULT:+$PG_BIN_DEFAULT/psql}}"
CREATEDB="${CREATEDB:-${PG_BIN_DEFAULT:+$PG_BIN_DEFAULT/createdb}}"
PG_RECVLOGICAL="${PG_RECVLOGICAL:-${PG_BIN_DEFAULT:+$PG_BIN_DEFAULT/pg_recvlogical}}"
PG_CTL="${PG_CTL:-pg_ctl}"
INITDB="${INITDB:-initdb}"
PSQL="${PSQL:-psql}"
CREATEDB="${CREATEDB:-createdb}"
PG_RECVLOGICAL="${PG_RECVLOGICAL:-pg_recvlogical}"

PUBLICATION="trellara_perf_pub"
SLOT="trellara_perf_slot"
PGURL="postgresql://$ROLE@localhost:$PORT/$DB"
CONFIG="$RUN_ROOT/flow.yml"
STREAM_DIR="$RUN_ROOT/local-stream"
SPILL_DIR="$RUN_ROOT/spill"
RESULT_JSON="$RUN_ROOT/result.json"
BOOTSTRAP_JSON="$RUN_ROOT/bootstrap.json"
DRAIN_STDOUT="$RUN_ROOT/drain.out"
DRAIN_TIME_LOG="$RUN_ROOT/drain-time.txt"

mkdir -p "$RUN_ROOT"

if [[ "$DRAIN_MODE" == "trellara" && ! -x "$TRELLARA_BIN" ]]; then
  echo "missing Trellara binary: $TRELLARA_BIN" >&2
  echo "build it first with: cargo build -p trellara-cli" >&2
  exit 1
fi

if [[ -n "$TARGET_BYTES" ]]; then
  target_bytes="$TARGET_BYTES"
elif [[ -n "$TARGET_MIB" ]]; then
  target_bytes=$((TARGET_MIB * 1024 * 1024))
else
  target_bytes=$((TARGET_GIB * 1024 * 1024 * 1024))
fi

payload_bytes_per_row=$((TX_BYTES / TABLE_COUNT))
if (( payload_bytes_per_row < 64 )); then
  payload_bytes_per_row=64
fi

estimated_target_tx=$((target_bytes / TX_BYTES))
if (( estimated_target_tx < 1 )); then
  estimated_target_tx=1
fi
if [[ -n "${CALIBRATION_TX:-}" ]]; then
  calibration_tx="$CALIBRATION_TX"
else
  calibration_tx=$((estimated_target_tx / 200))
  if (( calibration_tx < 100 )); then
    calibration_tx=100
  fi
  if (( calibration_tx > 10000 )); then
    calibration_tx=10000
  fi
fi

drain_pid=""
schema_change_enabled=false
pre_schema_tx_count=0
post_schema_tx_count=0
schema_change_ms=0
schema_change_lsn=""
schema_change_lsn_json=null
post_schema_marker_rows_json=null
expected_post_schema_marker_rows_json=null
schema_change_column_count_json=null
drain_continued_after_schema_change_json=null
schema_change_assertion_passed_json=null

case "$SCHEMA_CHANGE" in
  1|true|TRUE|yes|YES|on|ON)
    schema_change_enabled=true
    ;;
esac

if [[ "$SCHEMA_MARKER" == *"'"* ]]; then
  echo "SCHEMA_MARKER must not contain single quotes" >&2
  exit 1
fi
if [[ "$SCHEMA_MARKER" == *'"'* || "$SCHEMA_MARKER" == *"\\"* ]]; then
  echo "SCHEMA_MARKER must not contain double quotes or backslashes" >&2
  exit 1
fi
if [[ ! "$SCHEMA_CHANGE_AFTER_PERCENT" =~ ^[0-9]+$ ]]; then
  echo "SCHEMA_CHANGE_AFTER_PERCENT must be a whole number" >&2
  exit 1
fi
if (( SCHEMA_CHANGE_AFTER_PERCENT > 100 )); then
  echo "SCHEMA_CHANGE_AFTER_PERCENT must be between 0 and 100" >&2
  exit 1
fi
if [[ -n "$SCHEMA_CHANGE_AFTER_TX" && ! "$SCHEMA_CHANGE_AFTER_TX" =~ ^[0-9]+$ ]]; then
  echo "SCHEMA_CHANGE_AFTER_TX must be a whole number" >&2
  exit 1
fi

bytes_to_human() {
  awk -v bytes="$1" 'BEGIN {
    split("B KiB MiB GiB TiB", units, " ");
    value = bytes;
    unit = 1;
    while (value >= 1024 && unit < 5) {
      value /= 1024;
      unit++;
    }
    printf "%.2f %s", value, units[unit];
  }'
}

now_ms() {
  perl -MTime::HiRes=time -e 'printf "%d\n", time() * 1000'
}

rate_per_second() {
  awk -v value="$1" -v ms="$2" 'BEGIN {
    if (ms <= 0) {
      printf "0";
    } else {
      printf "%.2f", value * 1000 / ms;
    }
  }'
}

sql_scalar() {
  "$PSQL" "$PGURL" -XAtq -v ON_ERROR_STOP=1 -c "$1"
}

path_bytes() {
  local path="$1"
  if [[ -e "$path" ]]; then
    du -sk "$path" | awk '{ print $1 * 1024 }'
  else
    echo 0
  fi
}

archive_path() {
  local path="$1"
  if [[ -e "$path" ]]; then
    mv "$path" "$path.archive.$(date +%Y%m%d%H%M%S)"
  fi
}

stop_drain() {
  if [[ -n "$drain_pid" ]] && kill -0 "$drain_pid" 2>/dev/null; then
    kill "$drain_pid" 2>/dev/null || true
    wait "$drain_pid" 2>/dev/null || true
  fi
}

stop_postgres_if_requested() {
  if [[ "$KEEP_POSTGRES" == "0" ]]; then
    "$PG_CTL" -D "$PGDATA" stop -m fast >/dev/null 2>&1 || true
  fi
}

cleanup() {
  stop_drain
  stop_postgres_if_requested
}
trap cleanup EXIT

start_postgres() {
  if [[ ! -d "$PGDATA" ]]; then
    "$INITDB" -D "$PGDATA" -A trust >/dev/null
    {
      echo "listen_addresses = 'localhost'"
      echo "port = $PORT"
      echo "wal_level = logical"
      echo "max_replication_slots = 16"
      echo "max_wal_senders = 16"
      echo "max_slot_wal_keep_size = -1"
      echo "checkpoint_timeout = '5min'"
      echo "max_wal_size = '$MAX_WAL_SIZE'"
      echo "wal_sender_timeout = '0'"
    } >> "$PGDATA/postgresql.conf"
  fi

  if ! "$PG_CTL" -D "$PGDATA" status >/dev/null 2>&1; then
    "$PG_CTL" -D "$PGDATA" -l "$RUN_ROOT/postgres.log" start >/dev/null
  fi
}

create_role_and_database() {
  "$PSQL" "postgresql://localhost:$PORT/postgres" -Xq -v ON_ERROR_STOP=1 <<SQL
do \$\$
begin
  if not exists (select 1 from pg_roles where rolname = '$ROLE') then
    create role $ROLE login superuser;
  end if;
end
\$\$;
SQL
  if ! "$PSQL" "postgresql://localhost:$PORT/postgres" -XAtq -c \
    "select 1 from pg_database where datname = '$DB'" | grep -q 1; then
    "$CREATEDB" -h localhost -p "$PORT" -O "$ROLE" "$DB"
  fi
}

drop_slot_if_exists() {
  "$PSQL" "$PGURL" -Xq -v ON_ERROR_STOP=1 <<SQL
select pg_drop_replication_slot('$SLOT')
where exists (
  select 1
    from pg_replication_slots
   where slot_name = '$SLOT'
     and not active
);
SQL
}

prepare_workload_schema() {
  drop_slot_if_exists
  "$PSQL" "$PGURL" -Xq -v ON_ERROR_STOP=1 \
    -v payload_bytes="$payload_bytes_per_row" \
    -v pool_size="$PAYLOAD_POOL_SIZE" <<'SQL'
drop publication if exists trellara_perf_pub;
drop schema if exists perf_drain cascade;
create schema perf_drain;
create extension if not exists pgcrypto;

create table perf_drain.payload_pool (
  id integer primary key,
  payload text not null
);

insert into perf_drain.payload_pool (id, payload)
select g,
       left(
         repeat(encode(gen_random_bytes(1024), 'hex'), (:payload_bytes / 2048) + 2),
         :payload_bytes
       )
from generate_series(1, :pool_size) as g;

create table perf_drain.txn_table_1 (
  id bigint primary key,
  batch_id bigint not null,
  payload text not null,
  updated_at timestamptz not null default now()
);
create table perf_drain.txn_table_2 (like perf_drain.txn_table_1 including all);
create table perf_drain.txn_table_3 (like perf_drain.txn_table_1 including all);
create table perf_drain.txn_table_4 (like perf_drain.txn_table_1 including all);
create table perf_drain.txn_table_5 (like perf_drain.txn_table_1 including all);

alter table perf_drain.txn_table_1 alter column payload set storage external;
alter table perf_drain.txn_table_2 alter column payload set storage external;
alter table perf_drain.txn_table_3 alter column payload set storage external;
alter table perf_drain.txn_table_4 alter column payload set storage external;
alter table perf_drain.txn_table_5 alter column payload set storage external;

alter table perf_drain.txn_table_1 replica identity full;
alter table perf_drain.txn_table_2 replica identity full;
alter table perf_drain.txn_table_3 replica identity full;
alter table perf_drain.txn_table_4 replica identity full;
alter table perf_drain.txn_table_5 replica identity full;

create publication trellara_perf_pub for table
  perf_drain.txn_table_1,
  perf_drain.txn_table_2,
  perf_drain.txn_table_3,
  perf_drain.txn_table_4,
  perf_drain.txn_table_5;

create or replace procedure perf_drain.generate_small_transactions(
  start_tx bigint,
  tx_count bigint
)
language plpgsql
as $$
declare
  tx_id bigint;
  payloads text[];
  payload_count integer;
begin
  perform set_config('synchronous_commit', 'off', false);
  select array_agg(payload order by id)
    into payloads
    from perf_drain.payload_pool;
  payload_count := array_length(payloads, 1);

  for tx_id in start_tx..(start_tx + tx_count - 1) loop
    insert into perf_drain.txn_table_1 (id, batch_id, payload)
    values (tx_id, 1, payloads[((tx_id + 0) % payload_count) + 1]);
    insert into perf_drain.txn_table_2 (id, batch_id, payload)
    values (tx_id, 1, payloads[((tx_id + 1) % payload_count) + 1]);
    insert into perf_drain.txn_table_3 (id, batch_id, payload)
    values (tx_id, 1, payloads[((tx_id + 2) % payload_count) + 1]);
    insert into perf_drain.txn_table_4 (id, batch_id, payload)
    values (tx_id, 1, payloads[((tx_id + 3) % payload_count) + 1]);
    insert into perf_drain.txn_table_5 (id, batch_id, payload)
    values (tx_id, 1, payloads[((tx_id + 4) % payload_count) + 1]);
    commit;
  end loop;
end;
$$;
SQL
  archive_path "$STREAM_DIR"
  archive_path "$SPILL_DIR"
  mkdir -p "$STREAM_DIR" "$SPILL_DIR"
}

write_config() {
  cat > "$CONFIG" <<YAML
source:
  id: perf-source
  database_url: $PGURL
  publication: $PUBLICATION
  slot: $SLOT
  wal_retention_warn_bytes: 1073741824
  stream_spill_threshold_changes: 1024
  stream_spill_dir: $SPILL_DIR
  pgoutput:
    protocol_version: 2
    streaming: true

dataset:
  id: perf-drain
  mode: strict_transaction_order
  strict_chunking:
    max_changes_per_chunk: 1000
  tables:
    - schema: perf_drain
      name: txn_table_1
      verify:
        primary_key: id
        excluded_columns: [updated_at]
    - schema: perf_drain
      name: txn_table_2
      verify:
        primary_key: id
        excluded_columns: [updated_at]
    - schema: perf_drain
      name: txn_table_3
      verify:
        primary_key: id
        excluded_columns: [updated_at]
    - schema: perf_drain
      name: txn_table_4
      verify:
        primary_key: id
        excluded_columns: [updated_at]
    - schema: perf_drain
      name: txn_table_5
      verify:
        primary_key: id
        excluded_columns: [updated_at]

stream:
  kind: local
  path: $STREAM_DIR
  durability: fsync
YAML
}

generate_transactions() {
  local start_tx="$1"
  local tx_count="$2"
  if (( tx_count == 0 )); then
    return
  fi
  "$PSQL" "$PGURL" -Xq -v ON_ERROR_STOP=1 \
    -v start_tx="$start_tx" \
    -v tx_count="$tx_count" <<'SQL'
\timing on
call perf_drain.generate_small_transactions(:start_tx, :tx_count);
\timing off
SQL
}

apply_schema_change() {
  "$PSQL" "$PGURL" -Xq -v ON_ERROR_STOP=1 <<'SQL'
begin;

alter table perf_drain.txn_table_1 add column schema_revision text;
alter table perf_drain.txn_table_2 add column schema_revision text;
alter table perf_drain.txn_table_3 add column schema_revision text;
alter table perf_drain.txn_table_4 add column schema_revision text;
alter table perf_drain.txn_table_5 add column schema_revision text;

create or replace procedure perf_drain.generate_small_transactions_v2(
  start_tx bigint,
  tx_count bigint,
  schema_marker text
)
language plpgsql
as $$
declare
  tx_id bigint;
  payloads text[];
  payload_count integer;
begin
  perform set_config('synchronous_commit', 'off', false);
  select array_agg(payload order by id)
    into payloads
    from perf_drain.payload_pool;
  payload_count := array_length(payloads, 1);

  for tx_id in start_tx..(start_tx + tx_count - 1) loop
    insert into perf_drain.txn_table_1 (id, batch_id, payload, schema_revision)
    values (tx_id, 2, payloads[((tx_id + 0) % payload_count) + 1], schema_marker);
    insert into perf_drain.txn_table_2 (id, batch_id, payload, schema_revision)
    values (tx_id, 2, payloads[((tx_id + 1) % payload_count) + 1], schema_marker);
    insert into perf_drain.txn_table_3 (id, batch_id, payload, schema_revision)
    values (tx_id, 2, payloads[((tx_id + 2) % payload_count) + 1], schema_marker);
    insert into perf_drain.txn_table_4 (id, batch_id, payload, schema_revision)
    values (tx_id, 2, payloads[((tx_id + 3) % payload_count) + 1], schema_marker);
    insert into perf_drain.txn_table_5 (id, batch_id, payload, schema_revision)
    values (tx_id, 2, payloads[((tx_id + 4) % payload_count) + 1], schema_marker);
    commit;
  end loop;
end;
$$;

commit;
SQL
}

generate_transactions_v2() {
  local start_tx="$1"
  local tx_count="$2"
  if (( tx_count == 0 )); then
    return
  fi
  "$PSQL" "$PGURL" -Xq -v ON_ERROR_STOP=1 \
    -v start_tx="$start_tx" \
    -v tx_count="$tx_count" \
    -v schema_marker="$SCHEMA_MARKER" <<'SQL'
\timing on
call perf_drain.generate_small_transactions_v2(:start_tx, :tx_count, :'schema_marker');
\timing off
SQL
}

calculate_schema_split() {
  local explicit_split=false
  if [[ -n "$SCHEMA_CHANGE_AFTER_TX" ]]; then
    pre_schema_tx_count="$SCHEMA_CHANGE_AFTER_TX"
    explicit_split=true
  else
    pre_schema_tx_count=$((tx_count * SCHEMA_CHANGE_AFTER_PERCENT / 100))
  fi

  if (( tx_count <= 1 )); then
    pre_schema_tx_count=0
  else
    if [[ "$explicit_split" == "false" ]] && (( pre_schema_tx_count < 1 )); then
      pre_schema_tx_count=1
    fi
    if (( pre_schema_tx_count >= tx_count )); then
      pre_schema_tx_count=$((tx_count - 1))
    fi
  fi

  post_schema_tx_count=$((tx_count - pre_schema_tx_count))
}

count_post_schema_marker_rows() {
  "$PSQL" "$PGURL" -XAtq -v ON_ERROR_STOP=1 \
    -v schema_marker="$SCHEMA_MARKER" <<'SQL'
select coalesce(sum(row_count), 0)::bigint
from (
  select count(*) as row_count from perf_drain.txn_table_1 where schema_revision = :'schema_marker'
  union all
  select count(*) as row_count from perf_drain.txn_table_2 where schema_revision = :'schema_marker'
  union all
  select count(*) as row_count from perf_drain.txn_table_3 where schema_revision = :'schema_marker'
  union all
  select count(*) as row_count from perf_drain.txn_table_4 where schema_revision = :'schema_marker'
  union all
  select count(*) as row_count from perf_drain.txn_table_5 where schema_revision = :'schema_marker'
) counts;
SQL
}

count_schema_revision_columns() {
  "$PSQL" "$PGURL" -XAtq -v ON_ERROR_STOP=1 <<'SQL'
select count(*)::int
from information_schema.columns
where table_schema = 'perf_drain'
  and table_name in (
    'txn_table_1',
    'txn_table_2',
    'txn_table_3',
    'txn_table_4',
    'txn_table_5'
  )
  and column_name = 'schema_revision';
SQL
}

create_slot() {
  case "$DRAIN_MODE" in
    pg_recvlogical)
      "$PG_RECVLOGICAL" -d "$PGURL" -S "$SLOT" --create-slot -P pgoutput
      ;;
    trellara)
      "$TRELLARA_BIN" bootstrap --config "$CONFIG" > "$BOOTSTRAP_JSON"
      ;;
    *)
      echo "unsupported DRAIN_MODE=$DRAIN_MODE" >&2
      exit 1
      ;;
  esac
}

start_drain() {
  case "$DRAIN_MODE" in
    pg_recvlogical)
      /usr/bin/time -lp "$PG_RECVLOGICAL" \
        -d "$PGURL" \
        -S "$SLOT" \
        --start \
        -f /dev/null \
        -n \
        -s 1 \
        -o proto_version=2 \
        -o publication_names="$PUBLICATION" \
        > "$DRAIN_STDOUT" 2> "$DRAIN_TIME_LOG" &
      drain_pid="$!"
      sleep 1
      ;;
    trellara)
      /usr/bin/time -lp "$TRELLARA_BIN" relay --config "$CONFIG" --max-transactions "$tx_count" \
        > "$DRAIN_STDOUT" 2> "$DRAIN_TIME_LOG" &
      drain_pid="$!"
      sleep 1
      ;;
  esac
}

slot_lsn() {
  sql_scalar "select confirmed_flush_lsn::text from pg_replication_slots where slot_name = '$SLOT'"
}

wait_for_slot_lsn() {
  local target_lsn="$1"
  local deadline
  deadline=$(($(date +%s) + DRAIN_TIMEOUT_SECONDS))
  while true; do
    local caught_up
    caught_up="$(sql_scalar "select pg_wal_lsn_diff(confirmed_flush_lsn, '$target_lsn') >= 0 from pg_replication_slots where slot_name = '$SLOT'")"
    if [[ "$caught_up" == "t" ]]; then
      break
    fi
    if (( $(date +%s) >= deadline )); then
      echo "timed out waiting for slot $SLOT to reach $target_lsn" >&2
      return 1
    fi
    sleep 1
  done
}

relation_size_bytes() {
  sql_scalar "select coalesce(sum(pg_total_relation_size(format('%I.%I', schemaname, tablename)::regclass)), 0)::bigint from pg_tables where schemaname = 'perf_drain'"
}

main() {
  start_postgres
  create_role_and_database
  prepare_workload_schema
  write_config

  calibration_before_size="$(sql_scalar "select pg_database_size(current_database())")"
  generate_transactions 1 "$calibration_tx"
  calibration_after_size="$(sql_scalar "select pg_database_size(current_database())")"
  calibration_growth=$((calibration_after_size - calibration_before_size))
  bytes_per_tx=$((calibration_growth / calibration_tx))
  if (( bytes_per_tx < 1 )); then
    bytes_per_tx="$TX_BYTES"
  fi

  prepare_workload_schema
  write_config

  database_size_before="$(sql_scalar "select pg_database_size(current_database())")"
  if [[ -n "${TX_COUNT:-}" ]]; then
    tx_count="$TX_COUNT"
  elif (( target_bytes > database_size_before )); then
    tx_count=$(((target_bytes - database_size_before + bytes_per_tx - 1) / bytes_per_tx))
  else
    tx_count=1
  fi
  if (( tx_count < 1 )); then
    tx_count=1
  fi

  before_lsn="$(sql_scalar "select pg_current_wal_insert_lsn()::text")"
  pgdata_bytes_before="$(path_bytes "$PGDATA")"
  create_slot
  slot_lsn_before_generate="$(slot_lsn)"
  if [[ "$DRAIN_MODE" == "pg_recvlogical" ]]; then
    start_drain
  fi

  generate_start_ms="$(now_ms)"
  if [[ "$schema_change_enabled" == "true" ]]; then
    calculate_schema_split
    generate_transactions 1 "$pre_schema_tx_count"
    schema_change_start_ms="$(now_ms)"
    apply_schema_change
    schema_change_end_ms="$(now_ms)"
    schema_change_ms=$((schema_change_end_ms - schema_change_start_ms))
    schema_change_lsn="$(sql_scalar "select pg_current_wal_insert_lsn()::text")"
    schema_change_lsn_json="\"$schema_change_lsn\""
    generate_transactions_v2 "$((pre_schema_tx_count + 1))" "$post_schema_tx_count"
  else
    generate_transactions 1 "$tx_count"
  fi
  generate_end_ms="$(now_ms)"
  after_generate_lsn="$(sql_scalar "select pg_current_wal_insert_lsn()::text")"
  database_size_after_generate="$(sql_scalar "select pg_database_size(current_database())")"
  relation_size_after_generate="$(relation_size_bytes)"
  pgdata_bytes_after_generate="$(path_bytes "$PGDATA")"

  if [[ "$DRAIN_MODE" == "trellara" ]]; then
    start_drain
    drain_exit_status=0
    wait "$drain_pid" 2>/dev/null || drain_exit_status=$?
    drain_pid=""
    drain_caught_up_ms="$(now_ms)"
  else
    wait_for_slot_lsn "$after_generate_lsn"
    drain_caught_up_ms="$(now_ms)"
    stop_drain
    drain_exit_status=0
    drain_pid=""
  fi

  if [[ "$CHECKPOINT_AFTER_DRAIN" == "1" ]]; then
    "$PSQL" "$PGURL" -Xq -v ON_ERROR_STOP=1 -c "checkpoint" >/dev/null
  fi
  slot_lsn_after_drain="$(slot_lsn)"

  if [[ "$schema_change_enabled" == "true" ]]; then
    post_schema_marker_rows="$(count_post_schema_marker_rows)"
    expected_post_schema_marker_rows=$((post_schema_tx_count * TABLE_COUNT))
    schema_change_column_count="$(count_schema_revision_columns)"
    drain_continued_after_schema_change="$(
      sql_scalar "select pg_wal_lsn_diff('$after_generate_lsn', '$schema_change_lsn') > 0 and pg_wal_lsn_diff('$slot_lsn_after_drain', '$after_generate_lsn') >= 0"
    )"

    post_schema_marker_rows_json="$post_schema_marker_rows"
    expected_post_schema_marker_rows_json="$expected_post_schema_marker_rows"
    schema_change_column_count_json="$schema_change_column_count"
    if [[ "$drain_continued_after_schema_change" == "t" ]]; then
      drain_continued_after_schema_change_json=true
    else
      drain_continued_after_schema_change_json=false
    fi
    if [[ "$post_schema_marker_rows" == "$expected_post_schema_marker_rows" \
       && "$schema_change_column_count" == "$TABLE_COUNT" \
       && "$drain_continued_after_schema_change" == "t" ]]; then
      schema_change_assertion_passed_json=true
    else
      schema_change_assertion_passed_json=false
    fi
  fi

  retained_wal_bytes="$(sql_scalar "select coalesce(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn), 0)::bigint from pg_replication_slots where slot_name = '$SLOT'")"
  wal_generated_bytes="$(sql_scalar "select pg_wal_lsn_diff('$after_generate_lsn', '$before_lsn')::bigint")"
  stream_bytes="$(path_bytes "$STREAM_DIR")"
  spill_bytes="$(path_bytes "$SPILL_DIR")"
  pgdata_bytes_after_drain="$(path_bytes "$PGDATA")"
  generated_payload_bytes=$((tx_count * payload_bytes_per_row * TABLE_COUNT))
  requested_tx_bytes=$((tx_count * TX_BYTES))
  generate_ms=$((generate_end_ms - generate_start_ms))
  drain_ms=$((drain_caught_up_ms - generate_start_ms))
  catchup_ms=$((drain_caught_up_ms - generate_end_ms))
  tx_per_second="$(rate_per_second "$tx_count" "$drain_ms")"
  requested_bytes_per_second="$(rate_per_second "$requested_tx_bytes" "$drain_ms")"
  wal_bytes_per_second="$(rate_per_second "$wal_generated_bytes" "$drain_ms")"

  cat > "$RESULT_JSON" <<JSON
{
  "drain_mode": "$DRAIN_MODE",
  "target_database_bytes": $target_bytes,
  "target_database_human": "$(bytes_to_human "$target_bytes")",
  "target_transaction_bytes": $TX_BYTES,
  "table_count_per_transaction": $TABLE_COUNT,
  "payload_bytes_per_row": $payload_bytes_per_row,
  "payload_bytes_per_transaction": $((payload_bytes_per_row * TABLE_COUNT)),
  "payload_pool_size": $PAYLOAD_POOL_SIZE,
  "schema_change_enabled": $schema_change_enabled,
  "schema_change_marker": "$SCHEMA_MARKER",
  "schema_change_after_percent": $SCHEMA_CHANGE_AFTER_PERCENT,
  "schema_change_after_transactions": $pre_schema_tx_count,
  "post_schema_transactions": $post_schema_tx_count,
  "schema_change_ms": $schema_change_ms,
  "schema_change_lsn": $schema_change_lsn_json,
  "post_schema_marker_rows": $post_schema_marker_rows_json,
  "expected_post_schema_marker_rows": $expected_post_schema_marker_rows_json,
  "schema_change_column_count": $schema_change_column_count_json,
  "drain_continued_after_schema_change": $drain_continued_after_schema_change_json,
  "schema_change_assertion_passed": $schema_change_assertion_passed_json,
  "calibration_transactions": $calibration_tx,
  "calibration_growth_bytes": $calibration_growth,
  "calibrated_database_bytes_per_transaction": $bytes_per_tx,
  "transactions_generated": $tx_count,
  "requested_transaction_bytes_generated": $requested_tx_bytes,
  "requested_transaction_human_generated": "$(bytes_to_human "$requested_tx_bytes")",
  "payload_bytes_generated": $generated_payload_bytes,
  "payload_human_generated": "$(bytes_to_human "$generated_payload_bytes")",
  "postgres_url": "$PGURL",
  "postgres_data_dir": "$PGDATA",
  "config": "$CONFIG",
  "drain_stdout": "$DRAIN_STDOUT",
  "drain_time_log": "$DRAIN_TIME_LOG",
  "generate_ms": $generate_ms,
  "drain_ms_until_confirmed_flush": $drain_ms,
  "post_generate_catchup_ms": $catchup_ms,
  "drain_exit_status": $drain_exit_status,
  "transactions_per_second": $tx_per_second,
  "requested_transaction_bytes_per_second": $requested_bytes_per_second,
  "requested_transaction_human_per_second": "$(bytes_to_human "$requested_bytes_per_second")/s",
  "wal_bytes_per_second": $wal_bytes_per_second,
  "wal_human_per_second": "$(bytes_to_human "$wal_bytes_per_second")/s",
  "before_lsn": "$before_lsn",
  "slot_lsn_before_generate": "$slot_lsn_before_generate",
  "after_generate_lsn": "$after_generate_lsn",
  "slot_lsn_after_drain": "$slot_lsn_after_drain",
  "wal_generated_bytes": $wal_generated_bytes,
  "wal_generated_human": "$(bytes_to_human "$wal_generated_bytes")",
  "retained_wal_bytes_after_drain": $retained_wal_bytes,
  "retained_wal_human_after_drain": "$(bytes_to_human "$retained_wal_bytes")",
  "database_size_before_bytes": $database_size_before,
  "database_size_before_human": "$(bytes_to_human "$database_size_before")",
  "database_size_after_generate_bytes": $database_size_after_generate,
  "database_size_after_generate_human": "$(bytes_to_human "$database_size_after_generate")",
  "relation_size_after_generate_bytes": $relation_size_after_generate,
  "relation_size_after_generate_human": "$(bytes_to_human "$relation_size_after_generate")",
  "pgdata_bytes_before": $pgdata_bytes_before,
  "pgdata_bytes_after_generate": $pgdata_bytes_after_generate,
  "pgdata_bytes_after_drain": $pgdata_bytes_after_drain,
  "stream_bytes": $stream_bytes,
  "stream_human": "$(bytes_to_human "$stream_bytes")",
  "spill_bytes_after_drain": $spill_bytes,
  "spill_human_after_drain": "$(bytes_to_human "$spill_bytes")"
}
JSON

  echo "Postgres drain benchmark result: $RESULT_JSON"
  cat "$RESULT_JSON"

  if [[ "$schema_change_enabled" == "true" && "$schema_change_assertion_passed_json" != "true" ]]; then
    echo "schema-change drain assertion failed" >&2
    exit 1
  fi
}

main "$@"
