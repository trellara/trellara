#!/usr/bin/env bash
set -euo pipefail

workspace_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
pg_config=${TRELLARA_PG_CONFIG:?set TRELLARA_PG_CONFIG to PG15, PG16, PG17, or PG18 pg_config}
pg_major=$($pg_config --version | awk '{print $2}' | cut -d. -f1)
redpanda_image=${TRELLARA_REDPANDA_IMAGE:-docker.redpanda.com/redpandadata/redpanda:v26.2.2}
redpanda_port=${TRELLARA_REDPANDA_PORT:-19092}
container="trellara-redpanda-pg${pg_major}-$$"
test_root=$(mktemp -d "/tmp/trellara-kafka-pg${pg_major}.XXXXXX")
proof_path="$test_root/native-relay.kafka-proof"
topic="trellara.pg${pg_major}-live.smoke.strict"

cleanup() {
  docker rm -f "$container" >/dev/null 2>&1 || true
  if [[ -d "$test_root" ]]; then
    find "$test_root" -depth -delete
  fi
}
trap cleanup EXIT

docker run -d --name "$container" -p "127.0.0.1:${redpanda_port}:19092" \
  "$redpanda_image" redpanda start \
  --kafka-addr "internal://0.0.0.0:9092,external://0.0.0.0:19092" \
  --advertise-kafka-addr \
  "internal://${container}:9092,external://127.0.0.1:${redpanda_port}" \
  --mode dev-container --smp 1 --default-log-level=warn >/dev/null

wait_for_broker() {
  for _ in $(seq 1 120); do
    if docker exec "$container" rpk cluster info -X brokers=127.0.0.1:19092 \
      >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.25
  done
  docker logs "$container" >&2 || true
  echo "timed out waiting for Redpanda" >&2
  exit 1
}

wait_for_broker
TRELLARA_PG_CONFIG="$pg_config" \
TRELLARA_NATIVE_RELAY_DURABILITY=kafka \
TRELLARA_KAFKA_BOOTSTRAP_SERVERS="127.0.0.1:${redpanda_port}" \
TRELLARA_KAFKA_PROOF_PATH="$proof_path" \
  "$workspace_root/scripts/test-native-postgres-extension.sh"

[[ -s "$proof_path" ]] || {
  echo "Kafka publish-proof ledger is empty" >&2
  exit 1
}

docker stop --time 10 "$container" >/dev/null
docker start "$container" >/dev/null
wait_for_broker

keys=$(docker exec "$container" rpk topic consume "$topic" -n 8 \
  --format '%k\n' -X brokers=127.0.0.1:19092)
message_count=$(printf '%s\n' "$keys" | sed '/^$/d' | wc -l | tr -d ' ')
unique_keys=$(printf '%s\n' "$keys" | sed '/^$/d' | sort -u | wc -l | tr -d ' ')
[[ "$message_count" == 8 && "$unique_keys" == 8 ]] || {
  echo "expected eight durable unique Kafka transactions, got $message_count/$unique_keys" >&2
  exit 1
}

echo "PG${pg_major} -> native relay -> Kafka durability test passed"
