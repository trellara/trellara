# Development standards: trellara-lake

## Engineering goal

Turn replayable transaction boundaries into deterministic, complete, and recoverable lake commit plans without exposing partial epochs or duplicating data on replay.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Keep planning pure and separate from future object-store/catalog side effects.
- Use typed epochs, manifests, row intents, operations, and errors.
- Make ordering deterministic and use stable content-derived identifiers/digests.
- Use checked counts and byte sizes; reject unbounded or inconsistent metadata.
- Avoid panics and implicit defaults for visibility, gap, DDL, or recovery policy.
- Keep raw CDC facts immutable; derive current/SCD2 state through explicit plans.

## Lake guardrails

- Visibility requires complete manifests, commit markers, participating partitions, required metadata, and DDL gates.
- Exact replay may deduplicate; conflicting duplicate identity must fail closed.
- Never advance an epoch because downstream writes are merely planned rather than durably committed.
- Preserve source transaction identity, ordering, schema version, and operation type in raw CDC.
- Use deterministic file/table naming and commit-operation order.
- Recovery guidance must distinguish safe replay, quarantine, and manual intervention.

## Testing expectations

- Cover complete and incomplete epochs, gaps, duplicates, conflicts, missing metadata, and replay after partial work.
- Test raw/current/SCD2 plans from the same input boundary.
- Assert deterministic IDs, operation ordering, manifests, and recovery decisions.
- Add DDL acknowledgement tests whenever derived visibility rules change.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-lake --all-targets -- -D warnings
cargo test -p trellara-lake
cargo test -p trellara-lake --features parquet-writer
git diff --check
```
