#!/usr/bin/env bash
set -euo pipefail

workspace_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
pg_config=${TRELLARA_PG_CONFIG:?set TRELLARA_PG_CONFIG to PG15, PG16, PG17, or PG18 pg_config}
pg_bin=$(dirname "$pg_config")
pg_major=$($pg_config --version | awk '{print $2}' | cut -d. -f1)
package_root=${TRELLARA_PACKAGE_ROOT:-}
use_installed_extension=${TRELLARA_USE_INSTALLED_EXTENSION:-false}
extension_version=$(sed -n '/^\[package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' \
  "$workspace_root/crates/trellara-pg-extension/Cargo.toml")
test_root=$(mktemp -d "/tmp/trellara-pg${pg_major}-live.XXXXXX")
socket_path="$test_root/relay.sock"
spool_path="$test_root/relay.spool"
kafka_proof_path=${TRELLARA_KAFKA_PROOF_PATH:-"$test_root/relay.kafka-proof"}
relay_durability=${TRELLARA_NATIVE_RELAY_DURABILITY:-spool}
secret="trellara-live-pg${pg_major}-secret"
port=$((55400 + pg_major))
relay_pid=
server_started=false
run_as_postgres=(env)
database_user=$(id -un)

if [[ $(id -u) -eq 0 ]]; then
  run_as_postgres=(runuser -u postgres --)
  database_user=postgres
  chown postgres:postgres "$test_root"
fi

cleanup() {
  if [[ -n "$relay_pid" ]]; then
    kill -TERM "$relay_pid" 2>/dev/null || true
    wait "$relay_pid" 2>/dev/null || true
  fi
  if [[ "$server_started" == true ]]; then
    "${run_as_postgres[@]}" "$pg_bin/pg_ctl" -D "$test_root/data" \
      -m fast -t 10 -w stop || \
      "${run_as_postgres[@]}" "$pg_bin/pg_ctl" -D "$test_root/data" \
        -m immediate -t 10 -w stop || true
  fi
}
trap cleanup EXIT

if [[ "$use_installed_extension" != true && -z "$package_root" ]]; then
  package_root="$test_root/package"
  pgrx_package_args=(package --pg-config "$pg_config" --out-dir "$package_root")
  if [[ ${TRELLARA_PGRX_DEBUG:-false} == true ]]; then
    pgrx_package_args+=(--debug)
  fi
  (
    cd "$workspace_root/crates/trellara-pg-extension"
    cargo pgrx "${pgrx_package_args[@]}"
  )
fi

pkglibdir=$($pg_config --pkglibdir)
sharedir=$($pg_config --sharedir)
if [[ "$use_installed_extension" == true ]]; then
  library=$(find "$pkglibdir" -maxdepth 1 -type f \
    -name 'trellara_pg_extension.*' -print -quit)
  [[ -n "$library" && \
      -f "$sharedir/extension/trellara_pg_extension.control" && \
      -f "$sharedir/extension/trellara_pg_extension--${extension_version}.sql" ]] || {
    echo "installed Trellara extension files were not found" >&2
    exit 1
  }
else
  library=$(find "$package_root$pkglibdir" -maxdepth 1 -type f \
    -name 'trellara_pg_extension.*' -print -quit)
  [[ -n "$library" ]] || {
    echo "packaged Trellara extension library was not found" >&2
    exit 1
  }
  install -m 755 "$library" "$pkglibdir/$(basename "$library")"
  install -m 644 "$package_root$sharedir/extension/trellara_pg_extension.control" \
    "$sharedir/extension/trellara_pg_extension.control"
  install -m 644 \
    "$package_root$sharedir/extension/trellara_pg_extension--${extension_version}.sql" \
    "$sharedir/extension/trellara_pg_extension--${extension_version}.sql"
fi

relay_binary=${TRELLARA_RELAY_BINARY:-}
if [[ -z "$relay_binary" ]]; then
  cargo build --manifest-path "$workspace_root/Cargo.toml" --release \
    -p trellara-relay --bin trellara-native-relay
  relay_binary="${CARGO_TARGET_DIR:-$workspace_root/target}/release/trellara-native-relay"
fi
[[ -x "$relay_binary" ]] || {
  echo "native relay binary is not executable: $relay_binary" >&2
  exit 1
}

"${run_as_postgres[@]}" "$pg_bin/initdb" -D "$test_root/data" --no-locale -E UTF8
"${run_as_postgres[@]}" "$pg_bin/pg_ctl" -D "$test_root/data" \
  -l "$test_root/postgres.log" -o "-p $port -k $test_root -c listen_addresses='' \
  -c wal_level=logical -c max_replication_slots=4 -c max_worker_processes=8 \
  -c shared_preload_libraries=trellara_pg_extension \
  -c output_plugin_libraries=trellara_pg_extension \
  -c trellara.database_name=postgres \
  -c trellara.slot_name=trellara_native -c trellara.source_id=pg${pg_major}-live \
  -c trellara.dataset_id=smoke -c trellara.relay_socket_path=$socket_path \
  -c trellara.relay_secret=$secret -c trellara.queue_capacity=1 \
  -c trellara.worker_poll_milliseconds=50 -c trellara.relay_timeout_milliseconds=250" \
  -w start
server_started=true

psql=("${run_as_postgres[@]}" "$pg_bin/psql" -h "$test_root" -p "$port" -U "$database_user" -d postgres -v ON_ERROR_STOP=1)

runtime_queue_value() {
  "${psql[@]}" -Atc "SELECT trellara.runtime_status()->'queue'->>'$1'" 2>/dev/null || true
}

slot_flush_lsn() {
  "${psql[@]}" -Atc \
    "SELECT confirmed_flush_lsn::text FROM pg_replication_slots WHERE slot_name='trellara_native'"
}

wait_for_queue_state() {
  local field=$1 expected=$2
  for _ in $(seq 1 200); do
    [[ "$(runtime_queue_value "$field")" == "$expected" ]] && return 0
    sleep 0.05
  done
  echo "timed out waiting for queue $field=$expected" >&2
  exit 1
}

wait_for_queue_at_least() {
  local field=$1 minimum=$2 value
  for _ in $(seq 1 200); do
    value=$(runtime_queue_value "$field")
    (( value >= minimum )) && return 0
    sleep 0.05
  done
  echo "timed out waiting for queue $field >= $minimum" >&2
  exit 1
}

wait_for_worker_restart() {
  local old_pid=$1 current_pid current_starts
  for _ in $(seq 1 400); do
    current_pid=$(runtime_queue_value worker_pid)
    current_starts=$(runtime_queue_value worker_starts)
    if [[ -n "$current_pid" && "$current_pid" != "$old_pid" && \
          "$current_starts" =~ ^[0-9]+$ && "$current_starts" -ge 1 ]]; then
      return 0
    fi
    sleep 0.05
  done
  echo "timed out waiting for background-worker restart" >&2
  exit 1
}

wait_for_slot_empty() {
  local pending
  for _ in $(seq 1 400); do
    pending=$("${psql[@]}" -Atc \
      "SELECT count(*) FROM pg_logical_slot_peek_binary_changes('trellara_native', null, null)" \
      2>/dev/null || true)
    [[ "$pending" == 0 ]] && return 0
    sleep 0.05
  done
  echo "timed out waiting for crash-replayed logical slot to drain" >&2
  exit 1
}

start_relay() {
  local crash_after_sync=${1:-false}
  local relay_environment=(
    "TRELLARA_NATIVE_RELAY_SECRET=$secret"
    "TRELLARA_NATIVE_RELAY_SOCKET=$socket_path"
    "TRELLARA_NATIVE_RELAY_SPOOL=$spool_path"
    "TRELLARA_NATIVE_RELAY_DURABILITY=$relay_durability"
  )
  if [[ "$relay_durability" == kafka ]]; then
    : "${TRELLARA_KAFKA_BOOTSTRAP_SERVERS:?required for kafka durability}"
    relay_environment+=(
      "TRELLARA_KAFKA_BOOTSTRAP_SERVERS=$TRELLARA_KAFKA_BOOTSTRAP_SERVERS"
      "TRELLARA_KAFKA_CLIENT_ID=${TRELLARA_KAFKA_CLIENT_ID:-trellara-native-live-pg${pg_major}}"
      "TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS=${TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS:-30000}"
      "TRELLARA_KAFKA_PROOF_PATH=$kafka_proof_path"
    )
  fi
  if [[ "$crash_after_sync" == true ]]; then
    relay_environment+=("TRELLARA_NATIVE_RELAY_EXIT_AFTER_SYNC=1")
  fi
  "${run_as_postgres[@]}" env "${relay_environment[@]}" "$relay_binary" &
  started_relay_pid=$!
}

"${psql[@]}" <<'SQL'
CREATE EXTENSION trellara_pg_extension;
CREATE TABLE native_toast(id bigint PRIMARY KEY, payload text, value text);
ALTER TABLE native_toast ALTER COLUMN payload SET STORAGE EXTERNAL;
INSERT INTO native_toast VALUES (1, repeat('x', 100000), 'before');
CREATE TABLE native_origin(id bigint PRIMARY KEY, value text);
SELECT pg_create_logical_replication_slot('trellara_native', 'trellara_pg_extension');
UPDATE native_toast SET value = 'after' WHERE id = 1;
SELECT pg_logical_emit_message(false, 'trellara', 'nontransactional-message');
BEGIN;
SELECT pg_logical_emit_message(true, 'trellara', 'transactional-message');
COMMIT;
SELECT pg_replication_origin_create('trellara_echo');
SELECT pg_replication_origin_session_setup('trellara_echo');
INSERT INTO native_origin VALUES (1, 'remote-origin');
SELECT pg_replication_origin_session_reset();
CREATE TABLE native_smoke(id bigint PRIMARY KEY, value text);
ALTER TABLE native_smoke REPLICA IDENTITY FULL;
INSERT INTO native_smoke VALUES (1, 'inserted');
UPDATE native_smoke SET value = 'updated' WHERE id = 1;
DELETE FROM native_smoke WHERE id = 1;
INSERT INTO native_smoke VALUES (2, 'truncate-me');
TRUNCATE native_smoke;
BEGIN;
INSERT INTO native_smoke VALUES (3, 'aborted');
ROLLBACK;
SQL

decoded_count=$("${psql[@]}" -Atc \
  "SELECT count(*) FROM pg_logical_slot_peek_binary_changes('trellara_native', null, null)")
[[ "$decoded_count" == 8 ]] || {
  echo "expected eight committed DML/truncate/message frames, got $decoded_count" >&2
  exit 1
}
toast_marker_count=$("${psql[@]}" -Atc \
  "SELECT count(*) FROM pg_logical_slot_peek_binary_changes('trellara_native', null, null)
   WHERE encode(data, 'hex') LIKE '%000000190300000000%'")
(( toast_marker_count >= 1 )) || {
  echo "expected native_toast update to encode an unchanged-TOAST marker" >&2
  exit 1
}
toast_payload_length=$("${psql[@]}" -Atc \
  "SELECT length(payload) FROM native_toast WHERE id = 1")
[[ "$toast_payload_length" == 100000 ]] || {
  echo "TOAST payload changed after unrelated update" >&2
  exit 1
}
"${psql[@]}" -c "ALTER SYSTEM SET trellara.enabled = 'on'"
"${psql[@]}" -c "SELECT pg_reload_conf()"
wait_for_queue_state "queued_frames" 1
wait_for_queue_at_least "backpressure_events" 1
before_ack=$(slot_flush_lsn)

start_relay true
crash_relay_pid=$started_relay_pid
set +e
wait "$crash_relay_pid"
crash_status=$?
set -e
[[ "$crash_status" -eq 86 ]] || {
  echo "expected relay failure injection exit 86, got $crash_status" >&2
  exit 1
}
[[ "$(slot_flush_lsn)" == "$before_ack" ]] || {
  echo "slot advanced without a durable relay acknowledgement" >&2
  exit 1
}

start_relay false
relay_pid=$started_relay_pid
wait_for_queue_state "durable_frames" 8
wait_for_queue_state "queued_frames" 0
[[ "$(slot_flush_lsn)" != "$before_ack" ]] || {
  echo "slot did not advance after durable relay acknowledgement" >&2
  exit 1
}

worker_pid=$(runtime_queue_value worker_pid)
kill -KILL "$worker_pid"
wait_for_worker_restart "$worker_pid"
wait_for_slot_empty
if [[ "$relay_durability" == kafka && ! -s "$kafka_proof_path" ]]; then
  echo "Kafka durability mode did not record a durable publish proof" >&2
  exit 1
fi
echo "PG${pg_major} native extension live/crash smoke test passed"
