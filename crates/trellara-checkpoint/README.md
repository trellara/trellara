# trellara-checkpoint

`trellara-checkpoint` owns Trellara's durable progress and evidence model. It provides in-memory and PostgreSQL-backed stores for source/target checkpoints, transaction deduplication, snapshots, partitions, DDL barriers, quarantine, and validation history.

## Responsibilities

- Define checkpoint-store traits and typed checkpoint/evidence records.
- Enforce canonical LSNs, monotonic progress, transaction-key identity, and duplicate consistency.
- Persist source and target progress in PostgreSQL with a versioned schema.
- Track snapshot runs, per-table progress, handoff readiness, and validation events.
- Track partition watermarks, lag summaries, and DDL acknowledgement/release evidence across target-Postgres, raw-CDC-lake, and Spark-derived-view sinks.
- Track Iceberg table-commit intents and receipts for lake-writer recovery.
- Record quarantine, reseed, snapshot-handoff, and apply-validation evidence.
- Provide a deterministic in-memory store for tests and simulations.

## PostgreSQL schema

`postgres_checkpoint_schema_sql()` returns the full DDL for the `trellara` schema. Thirteen tables
carry every durable fact the system is allowed to rely on:

| Table | Purpose |
| --- | --- |
| `trellara.flow_checkpoints` | Source and target progress per flow identity. |
| `trellara.applied_transactions` | Target-side transaction deduplication. |
| `trellara.partition_checkpoints` | Per-partition watermarks for partitioned scale mode. |
| `trellara.snapshot_runs` | Snapshot run identity and state machine. |
| `trellara.snapshot_table_progress` | Per-relation copy progress within a run. |
| `trellara.snapshot_handoff_events` | The recorded consistent LSN that CDC resumes from. |
| `trellara.ddl_barriers` | Schema barriers, policy digests, and release gates. |
| `trellara.ddl_barrier_acks` | Per-sink acknowledgement evidence for a barrier. |
| `trellara.apply_quarantine` | Blocked transactions and their failure evidence. |
| `trellara.reseed_events` | Explicit repair of a relation and its handoff evidence. |
| `trellara.validation_events` | Verification results and digests. |
| `trellara.iceberg_commit_intents` | Pre-commit writer intents for reconciliation. |
| `trellara.iceberg_commit_receipts` | Proven table-commit receipts. |

`InMemoryCheckpointStore` must enforce the same observable invariants as `PostgresCheckpointStore`;
tests that exist for one path should exist for the other.

## Boundaries

This crate records validated state; it does not publish, capture, apply target rows, compare table contents, or choose operator workflows. Callers own orchestration and must respect the store's monotonicity and atomicity guarantees.

PostgreSQL schema changes require migration/compatibility review and tests for existing records.

## Key entry points

- Checkpoint-store traits exported from `types`
- `InMemoryCheckpointStore` and `PostgresCheckpointStore`
- Snapshot, partition-watermark, DDL-barrier, and evidence types
- `postgres_checkpoint_schema_sql`

## Development

```console
cargo test -p trellara-checkpoint
cargo clippy -p trellara-checkpoint --all-targets -- -D warnings
```

See [skills.md](skills.md) and the [workspace map](../README.md).
