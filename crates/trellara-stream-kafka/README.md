# trellara-stream-kafka

`trellara-stream-kafka` implements Trellara's transport-neutral stream traits with Kafka-compatible brokers such as Apache Kafka and Redpanda.

## Responsibilities

- Configure and construct Kafka publishers and consumers.
- Convert `StreamMessage` values to and from Kafka records without changing metadata semantics.
- Map broker delivery results and offsets to Trellara publish acknowledgements and stream positions.
- Surface broker failures as typed `KafkaStreamError` values.
- Define the versioned production security, quorum, producer, and consumer-progress contract without silently applying it to local development configurations.

## Boundaries

This crate is an adapter. Topic/key/header rules belong in `trellara-stream`, transaction construction and source acknowledgement belong in `trellara-relay`, and operator configuration belongs in `trellara-cli`.

Do not hide broker durability settings or claim a durable acknowledgement before the configured Kafka acknowledgement policy succeeds.

## Key entry points

- `KafkaPublisher` and `KafkaPublisherConfig`
- `KafkaConsumer` and `KafkaConsumerConfig`
- `KafkaStreamError`
- `KafkaProductionContract`, `KafkaAuthenticationContract`, `KafkaSecurityConfig`, and `KafkaSecretRef`
- `KafkaTopologyRequirement` and `KafkaPublishIdentity`

`KafkaProductionContract::sasl_tls` is the locked production target: TLS, referenced credentials rather than inline secrets, replication factor at least three, minimum in-sync replicas at least two, `acks=all`, producer idempotence, disabled consumer auto-commit, and synchronous offset commit only after target apply.

Use `KafkaPublisherConfig::production` and `KafkaConsumerConfig::production` to map a validated contract into runtime configuration. Secret references are resolved only while constructing the librdkafka client. Environment references provide the secret contents; file references point to UTF-8 secret mounts whose contents are read at startup. Resolved values are not retained in the public config types or included in adapter errors.

The production consumer validates all subscribed topics before joining the group. The production publisher validates each topic before its first publish. Validation requires the configured number of distinct brokers and, for every partition, a healthy leader, distinct replicas, and the configured live ISR quorum. A topic that is absent, under-replicated, or below quorum is rejected before Trellara uses it as a durable boundary.

Consumers use `cooperative-sticky` assignment, retain state for partitions that remain assigned, synchronously commit only the contiguous applied prefix, and never commit after librdkafka reports that the assignment was lost. `max_in_flight_messages` bounds unapplied records; reaching the bound pauses the current assignment until acknowledgements reduce pressure.

`KafkaPublisher::publish_reconcilable` distinguishes records rejected before enqueue from delivery outcomes that may have reached the broker. An ambiguous result contains a `KafkaPublishIdentity`; compare it with consumed records using `KafkaPublishIdentity::matches` before advancing source progress. Replaying the same Trellara transaction remains safe because transaction identity is stable, but downstream apply must continue to deduplicate replay.

## Development

```console
cargo test -p trellara-stream-kafka
cargo clippy -p trellara-stream-kafka --all-targets -- -D warnings
```

Unit tests should remain broker-free. End-to-end transport tests require an explicitly configured Kafka/Redpanda service. See [skills.md](skills.md) and the [workspace map](../README.md).
