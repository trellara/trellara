# Development standards: trellara-stream-local

## Engineering goal

Provide a simple durable log whose acknowledgements, recovery, and replay behavior remain correct across process crashes, torn writes, corrupt bytes, and administrative cursor changes.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Use checked offset/length arithmetic and explicit maximum frame sizes.
- Treat all on-disk bytes and paths as untrusted. Validate before allocation, decoding, or path construction.
- Keep filesystem operations recoverable and errors typed with relevant topic/offset context.
- Avoid panics, unchecked slices, implicit truncation, and whole-file reads for unbounded logs.
- Isolate blocking filesystem work from async executors when APIs are used in asynchronous paths.
- Keep frame, index, cursor, inspection, and barrier-reconstruction modules independently testable.

## Durability guardrails

- Emit a durable `PublishAck` only after the configured frame and index durability boundary completes.
- Make append and cursor persistence crash-safe; temporary/partial files must be recognizable and recoverable.
- Cursors advance monotonically during normal acknowledgement. Backward movement requires an explicit seek policy.
- Rebuild indexes from validated frames and stop at corrupt/torn tails; never index unverifiable bytes.
- Prevent path traversal and cross-topic cursor or index confusion.
- Reject duplicate/conflicting barrier headers and incomplete reconstruction.

## Testing expectations

- Use temporary directories and deterministic bytes; never depend on a developer's local stream directory.
- Test torn tails at every header/body boundary, checksum corruption, oversized frames, and index rebuild.
- Test crash windows around append, sync, index update, cursor replace, and acknowledgement.
- Cover forward/backward seek policy and live-reader reconciliation.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-stream-local --all-targets -- -D warnings
cargo test -p trellara-stream-local
git diff --check
```
