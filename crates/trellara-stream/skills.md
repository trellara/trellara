# Development standards: trellara-stream

## Engineering goal

Keep the stream port minimal, deterministic, and implementable by transports with different operational models without weakening Trellara's durability contract.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Design traits around domain capabilities, not a particular client library's API.
- Use typed messages, positions, headers, acknowledgements, and errors.
- Keep public exports deliberate and document the guarantees callers may rely on.
- Avoid hidden allocation, cloning, or payload re-encoding on hot paths when borrowing is practical.
- Avoid panics and unbounded input processing; validate lengths and required metadata.
- Keep async traits cancellation-safe and free of blocking filesystem or network calls.

## Stream guardrails

- A publish acknowledgement must mean the durability level documented by the implementation.
- Cursor acknowledgement must be monotonic unless an explicit administrative seek API is used.
- Reject missing, duplicate, or conflicting barrier headers before downstream reconstruction.
- Keep topic and key generation deterministic across transports.
- Do not import Kafka-, filesystem-, PostgreSQL-, or CLI-specific types into the common interface.

## Testing expectations

- Add contract tests for new message validation and header rules.
- Test exact duplicates separately from conflicting duplicates.
- Test trait behavior through at least one concrete adapter when semantics change.
- Cover empty payloads, maximum sizes, invalid positions, and malformed metadata.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-stream --all-targets -- -D warnings
cargo test -p trellara-stream
git diff --check
```
