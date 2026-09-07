# Development standards: trellara-pg-capture

## Engineering goal

Turn an untrusted PostgreSQL replication byte stream into deterministic, validated, committed Trellara transactions without acknowledging data before downstream durability is proven.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Parse incrementally with explicit bounds; reject truncated, oversized, unknown, or internally inconsistent frames.
- Use typed state machines and errors for replication and transaction assembly.
- Avoid panics, unchecked slicing, lossy numeric conversions, unbounded buffers, and implicit UTF-8 assumptions.
- Keep async code cancellation-safe. Never block a Tokio worker with synchronous network or large filesystem operations.
- Redact credentials and connection strings from errors, tracing, fixtures, and debug output.
- Keep public APIs narrow and document ownership, lifetime, retry, and acknowledgement expectations.

## Capture guardrails

- Emit downstream visibility only at a valid commit boundary; abort must clean all transaction and spill state.
- Preserve source order, relation identity, schema version, transaction identity, and commit LSN exactly.
- Replication feedback may report only a caller-provided durable acknowledgement, never merely the last decoded LSN.
- Bound memory and spill-file growth; clean spill artifacts on success, abort, and recoverable failure.
- Fail closed on missing relation metadata, unsafe replica identity, slot/plugin mismatches, and subscription conflicts.
- Keep broker and target behavior out of the capture state machine.

## Testing expectations

- Use byte-level fixtures for valid, truncated, malformed, and oversized replication frames.
- Cover DML, truncate, DDL, streaming commit, streaming abort, subtransactions, schema drift, and relation-cache invalidation.
- Add crash/cleanup tests for spill and partial transaction state.
- Keep unit tests deterministic and independent of a running PostgreSQL server.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-pg-capture --all-targets -- -D warnings
cargo test -p trellara-pg-capture
git diff --check
```

## Source-safety query contract

- Treat `source_safety_read_only_queries` and the safety-inspection types as a versioned external contract: `trellara-check` depends on their exact shape and read-only behavior.
- Never add a write query to the manifest, and never widen a query's result shape without updating both consumers.
