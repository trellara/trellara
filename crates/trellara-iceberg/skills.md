# Development standards: trellara-iceberg

## Engineering goal

Commit complete Trellara epochs to Iceberg deterministically and idempotently without publishing partial cross-table visibility or weakening source acknowledgement boundaries.

## Rust standards

- Preserve the workspace Rust 1.97.1 minimum; keep optional third-party runtimes behind explicit features and document their toolchain floor.
- Keep commit planning and receipt validation pure, deterministic, serializable, and independently testable.
- Use typed table identifiers, completed-file evidence, commit plans, receipts, and errors.
- Prefer checked arithmetic and fallible conversion at external and persisted boundaries.
- Return actionable errors for invalid plans, catalog conflicts, and unsupported table layouts; do not panic on catalog input.
- Keep dependencies minimal and use workspace dependencies where available.

## Iceberg guardrails

- Never contact a catalog until the Trellara lake consumer gate accepts the epoch and every planned file has matching completion evidence.
- Treat the object key, URI, format, size, row count, relation, source bucket, and checksum rollup as commit evidence, not hints.
- Use deterministic commit identities and validate all lineage properties when replay finds an existing snapshot.
- Require duplicate-file checking and preserve Iceberg's optimistic concurrency requirements.
- Do not model multiple Iceberg table commits as atomic. Withhold Trellara epoch metadata until every table receipt matches the same epoch commit.
- Fail closed for partition specs, delete files, schema evolution, or catalog behavior the adapter does not explicitly support. The runtime supports exactly one raw partition spec — `identity(epoch_id), identity(source_bucket)` — and unpartitioned support tables; anything else is rejected, not approximated.
- Treat the raw CDC column list, its field IDs, and the derived SHA-256 schema fingerprint as a compatibility surface. Adding, reordering, or retyping a column changes the fingerprint and requires DDL acknowledgement evidence, not a silent evolution.
- Keep the metadata commit order (`10` sources, `20` tables, `30` partitions, `40` quarantine, `50` verification, `100` completeness) intact; `_trellara_epochs` last is the whole cross-table gate.
- Never move the source ACK boundary from durable stream publication to a slow or unavailable catalog.

## Testing expectations

- Cover deterministic planning, file reordering, missing/extra/conflicting files, incomplete and complete-with-gaps epochs, duplicate mappings, replay, and conflicting catalog evidence.
- Add tests for every supported partition transform and data-file content type before enabling it.
- Exercise catalog implementations with concurrent commit, ambiguous-success retry, and stale-snapshot scenarios before production enablement.
- Keep default unit tests free of catalog, object-store, network, and credential requirements.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-iceberg --all-targets -- -D warnings
cargo test -p trellara-iceberg
cargo check -p trellara-iceberg --features iceberg-rust
cargo check -p trellara-iceberg --features production-writer
git diff --check
```
