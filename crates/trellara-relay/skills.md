# Development standards: trellara-relay

## Engineering goal

Make the publish/checkpoint/source-acknowledgement sequence explicit, replay-safe, and provable under crashes and ambiguous transport outcomes.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Model relay phases and evidence with typed states and errors rather than loosely related flags.
- Keep async operations cancellation-safe; avoid holding locks or mutable borrows across network awaits.
- Use structured tracing with source, transaction, LSN, topic, and partition identifiers; never log payload secrets.
- Avoid panics and unbounded retries. Retry policies need limits, backoff, observability, and a safe replay basis.
- Keep orchestration separate from message construction, publishing, checkpoint persistence, and statistics.

## Relay guardrails

- Required order is: construct complete boundary, durably publish every message, record durable source checkpoint, then acknowledge source LSN.
- Treat missing or ambiguous publish acknowledgements as not durable until reconciliation proves otherwise.
- Validate expected and actual destinations, checksums, LSN coverage, and checkpoint ordering.
- Preserve deterministic topic, partition, key, and publish order for every relay mode.
- Never acknowledge a manifest or chunk subset as a complete transaction.
- Native feedback must prove the same durable boundary as client-side PostgreSQL feedback.
- Native worker reports must distinguish sleep, proven feedback readiness, and fail-closed rejection without turning errors into success.

## Testing expectations

- Add failure injection before/after publish, checkpoint, and source acknowledgement.
- Cover exact replay, conflicting evidence, missing destinations, partial barriers, and ambiguous publishes.
- Assert call ordering with recording fixtures, not only final counters.
- Test every relay mode when common acknowledgement logic changes.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-relay --all-targets -- -D warnings
cargo test -p trellara-relay
cargo check -p trellara-relay --features kafka
git diff --check
```
