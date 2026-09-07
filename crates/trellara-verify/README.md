# trellara-verify

`trellara-verify` proves whether a PostgreSQL source and target agree. It creates canonical relation snapshots, compares row identities and hashes, reports drift, and performs explicit reseed operations.

## Responsibilities

- Inspect PostgreSQL relation metadata needed for verification.
- Snapshot rows with deterministic primary-key ordering and canonical JSON hashing.
- Compare source and target snapshots and classify missing, extra, and mismatched rows.
- Report source/target watermarks and bounded drift samples.
- Generate and execute explicit PostgreSQL reseed operations.
- Import/export snapshot-compatible types used by operator workflows.

## Boundaries

Verification observes and repairs under explicit instruction; it does not participate in the normal apply transaction, advance stream cursors, or declare source durability. Checkpoint evidence is recorded by callers through `trellara-checkpoint`.

Canonicalization is a compatibility contract: equivalent PostgreSQL values must hash identically, and different identities must not collapse silently.

## Key entry points

- `snapshot_postgres_table`
- `compare_snapshots` and comparison report types
- `ensure_target_caught_up`
- `inspect_postgres_relation`
- `reseed_postgres_table`
- `RowSnapshot`

## Development

```console
cargo test -p trellara-verify
cargo clippy -p trellara-verify --all-targets -- -D warnings
```

See [skills.md](skills.md) and the [workspace map](../README.md).
