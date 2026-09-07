# trellara-stream

`trellara-stream` defines the transport-neutral boundary between relays, durable streams, and consumers. It lets local disk and Kafka/Redpanda implementations share the same message and acknowledgement semantics.

## Responsibilities

- Define `StreamPublisher` and `StreamConsumer` traits.
- Define messages, positions, headers, publish acknowledgements, and stream errors.
- Define topic layouts and stable message-key helpers for strict and partitioned modes.
- Validate common barrier headers and message metadata before adapters act on them.

## Boundaries

This crate does not implement disk files, Kafka clients, retry loops, source checkpoints, or target apply. Those belong in `trellara-stream-local`, `trellara-stream-kafka`, `trellara-relay`, and target packages respectively.

The traits describe observable durability and cursor behavior; adapters must not claim stronger guarantees than they actually provide.

## Key entry points

- `StreamPublisher`, `StreamConsumer`, and `PublishAck`
- `StreamMessage`, `StreamHeader`, and `StreamPosition`
- `StreamMode` and `TopicLayout`
- Barrier-header and deterministic key helpers

## Development

```console
cargo test -p trellara-stream
cargo clippy -p trellara-stream --all-targets -- -D warnings
```

See [skills.md](skills.md) and the [workspace map](../README.md).
