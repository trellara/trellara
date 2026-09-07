# Development standards: trellara-stream-kafka

## Engineering goal

Translate Trellara stream semantics to Kafka precisely, with honest durability, stable metadata, bounded retries, and observable broker failures.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Keep rdkafka types at the adapter boundary; expose common `trellara-stream` types to callers.
- Use typed configuration and errors. Validate required brokers, topics, groups, and durability settings early.
- Keep async operations cancellation-safe and avoid blocking callbacks.
- Avoid unnecessary payload copies and lossy header conversion.
- Redact credentials, SASL material, certificates, and full connection strings from logs and errors.
- Use structured tracing with safe topic, partition, offset, and correlation identifiers.

## Kafka guardrails

- Map delivery acknowledgements to the exact topic/partition/offset returned by the broker.
- Preserve binary keys, payloads, and duplicate headers without implicit text conversion; common validation decides which duplicates are legal.
- Do not auto-commit consumer progress before target-side processing succeeds.
- Retry only replay-safe operations and make ambiguous outcomes visible to the relay.
- Keep producer idempotence, acknowledgement, timeout, and ordering settings explicit.
- Preserve production topology validation and publish-identity reconciliation; a broker acknowledgement is not sufficient when the delivery outcome is ambiguous.
- Never encode domain rules that would behave differently in the local adapter.

## Testing expectations

- Unit-test message/header/position conversion and every broker error mapping without a live broker.
- Add opt-in integration coverage for publish, consume, redelivery, partition ordering, and broker restart.
- Cover binary headers, empty payloads, duplicate/conflicting metadata, timeouts, and ambiguous delivery.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-stream-kafka --all-targets -- -D warnings
cargo test -p trellara-stream-kafka
cargo check -p trellara-stream-kafka --no-default-features
git diff --check
```
