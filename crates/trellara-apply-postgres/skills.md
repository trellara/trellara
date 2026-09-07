# Development standards: trellara-apply-postgres

## Engineering goal

Make at-least-once delivery produce idempotent, atomic target effects while failing closed on incomplete barriers, unsafe SQL, schema drift, and checkpoint inconsistencies.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Separate validation, planning, execution, checkpointing, and worker orchestration.
- Use typed SQL values and parameter binding; never interpolate row values into SQL text.
- Validate and quote identifiers through centralized helpers. Treat DDL text as untrusted input.
- Avoid panics and partial mutation. Return typed errors with safe transaction/relation context.
- Keep async database operations cancellation-aware; avoid holding unrelated locks across awaits.
- Redact row data and connection secrets from logs unless an explicitly safe diagnostic representation exists.

## Apply guardrails

- Target mutation, dedup record, checkpoint, and required evidence for one transaction must commit atomically.
- On failure or quarantine, target checkpoint and dedup state must not advance.
- An exact redelivery must skip mutation safely and still permit stream acknowledgement after durable state is confirmed.
- Never apply a partial manifest/chunk barrier or release post-DDL DML before all required valid acknowledgements.
- Validate statement, plan, schema, and evidence digests before executing DDL.
- Reject affected-row mismatches unless an explicit policy makes them safe.

## Testing expectations

- Test SQL plans separately from database execution.
- Cover insert/update/delete/truncate, target-owned columns, nulls, key changes, and affected-row mismatches.
- Inject failures before/after mutation, dedup, checkpoint, and acknowledgement.
- Cover exact/conflicting duplicates, missing chunks, invalid headers, stale DDL ACKs, and digest tampering.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-apply-postgres --all-targets -- -D warnings
cargo test -p trellara-apply-postgres
git diff --check
```
