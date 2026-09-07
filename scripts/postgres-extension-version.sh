#!/usr/bin/env bash
set -euo pipefail

action=${1:?usage: postgres-extension-version.sh upgrade|rollback TARGET_VERSION CONNECTION}
target_version=${2:?usage: postgres-extension-version.sh upgrade|rollback TARGET_VERSION CONNECTION}
connection=${3:?usage: postgres-extension-version.sh upgrade|rollback TARGET_VERSION CONNECTION}

[[ "$action" == upgrade || "$action" == rollback ]] || {
  echo "action must be upgrade or rollback" >&2
  exit 2
}
[[ "$target_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][A-Za-z0-9]+)*$ ]] || {
  echo "target version has an invalid format" >&2
  exit 2
}
if [[ "$action" == rollback && ${TRELLARA_ALLOW_ROLLBACK:-} != 1 ]]; then
  echo "rollback requires TRELLARA_ALLOW_ROLLBACK=1" >&2
  exit 2
fi

psql_base=(psql "$connection" -X -v ON_ERROR_STOP=1)
current_version=$("${psql_base[@]}" -Atc \
  "SELECT extversion FROM pg_extension WHERE extname='trellara_pg_extension'")
[[ -n "$current_version" ]] || {
  echo "trellara_pg_extension is not installed in the target database" >&2
  exit 1
}
[[ "$current_version" != "$target_version" ]] || {
  echo "trellara_pg_extension is already at $target_version"
  exit 0
}

queued_frames=$("${psql_base[@]}" -Atc \
  "SELECT (trellara.runtime_status()->'queue'->>'queued_frames')::bigint")
[[ "$queued_frames" == 0 ]] || {
  echo "refusing $action while $queued_frames native frames remain queued" >&2
  exit 1
}

available_path=$("${psql_base[@]}" -At \
  -v source_version="$current_version" -v target_version="$target_version" \
  -c "SELECT path FROM pg_extension_update_paths('trellara_pg_extension') WHERE source=:'source_version' AND target=:'target_version'")
[[ -n "$available_path" ]] || {
  echo "no PostgreSQL extension path exists from $current_version to $target_version" >&2
  exit 1
}

"${psql_base[@]}" -v target_version="$target_version" <<'SQL'
BEGIN;
LOCK TABLE pg_catalog.pg_extension IN SHARE ROW EXCLUSIVE MODE;
ALTER EXTENSION trellara_pg_extension UPDATE TO :'target_version';
SELECT extname, extversion
FROM pg_catalog.pg_extension
WHERE extname = 'trellara_pg_extension';
COMMIT;
SQL
echo "trellara_pg_extension $action completed: $current_version -> $target_version"
