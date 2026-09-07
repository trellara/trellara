# trellara-relay

`trellara-relay` orchestrates the correctness boundary between source capture and durable stream publication. It plans messages, publishes them, records source progress, and acknowledges the source only after durable evidence is complete.

## Responsibilities

- Consume committed envelopes from a `PgChangeSource`.
- Publish strict, strict-chunked, or partitioned transaction messages in the required order.
- Require durable publish acknowledgements for every planned destination.
- Record the durable source checkpoint before advancing source acknowledgement.
- Produce machine-checkable source-ACK proofs and run statistics.
- Adapt relay publish evidence into native-extension feedback decisions.
- Run and report one bounded native-worker handoff iteration with fail-closed outcomes.
- Accept authenticated native frames over a mode-0600 Unix socket.
- Fsync a bounded frame into the recovery spool before publication.
- In Kafka mode, require an `acks=all` broker delivery result and fsync its cluster/topic/partition/offset proof before returning an authenticated ACK.
- Treat exact replays as durable duplicates and reject conflicting evidence at the same LSN.
- Handle ambiguous publish outcomes through safe replay and idempotency boundaries.

## Boundaries

The relay does not decode pgoutput, implement a broker, apply target SQL, or define protocol types. Those responsibilities live in capture, stream-adapter, apply, and protocol crates.

The relay may orchestrate retries, but it must not reinterpret a transport acknowledgement or weaken the durable-publish boundary.

The `trellara-native-relay` binary is the native extension's durable ingress. Configure it with `TRELLARA_NATIVE_RELAY_SECRET`, `TRELLARA_NATIVE_RELAY_SOCKET`, and `TRELLARA_NATIVE_RELAY_SPOOL`. Set `TRELLARA_NATIVE_RELAY_DURABILITY=kafka`, `TRELLARA_KAFKA_BOOTSTRAP_SERVERS`, and `TRELLARA_KAFKA_PROOF_PATH` to make Kafka delivery—not the local spool—the PostgreSQL acknowledgement boundary. The Linux package installs a disabled-by-default systemd unit and `/etc/trellara/native-relay.env`; set the secret and durability mode before enabling it.

Kafka mode publishes each native logical transaction to partition 0 of the deterministic strict topic with its transaction identity as the key. The producer uses idempotence and `acks=all`. The relay records the broker cluster ID, topic, partition, offset, commit LSN, and payload digest in an fsynced proof ledger. A matching replay returns the prior proof; a changed cluster, destination, or digest fails closed. An ambiguous publish without a recorded proof is replayed and can therefore produce a duplicate, which downstream consumers must deduplicate by transaction identity.

## Key entry points

- `Relay`, `RelayMode`, `RelayStep`, and `RelayRunStats`
- `SourceAckBoundaryProof` and `SOURCE_ACK_BOUNDARY_CONTRACT`
- Native feedback proof/decision adapters
- Native worker run outcomes and reports
- `run_native_relay`, `run_native_kafka_relay`, and `trellara-native-relay`
- `load_source_checkpoint`

## Development

```console
cargo test -p trellara-relay
cargo clippy -p trellara-relay --all-targets -- -D warnings
TRELLARA_PG_CONFIG=/path/to/pg_config scripts/test-native-kafka-durability.sh
```

See [skills.md](skills.md) and the [workspace map](../README.md).
