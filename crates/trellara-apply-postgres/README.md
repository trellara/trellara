# trellara-apply-postgres

`trellara-apply-postgres` is the PostgreSQL target adapter. It validates stream transactions, plans safe SQL, applies changes atomically, and records deduplication, checkpoint, quarantine, and DDL-barrier evidence.

## Responsibilities

- Plan parameterized INSERT, UPDATE, DELETE, and TRUNCATE operations from protocol changes.
- Apply complete envelopes atomically with target checkpoint and transaction deduplication state.
- Preserve configured target-owned columns and unchanged-TOAST values, and validate affected-row expectations.
- Reject and quarantine rows whose key columns arrive as unchanged TOAST rather than guess an identity.
- Reconstruct and apply strict-chunked or partitioned barrier transactions only when complete.
- Quarantine invalid or unsafe work without advancing target progress.
- Validate DDL policy, statement/plan digests, target acknowledgement evidence, and post-DDL release gates.
- Expose strict and barrier-aware apply workers with replay-safe statistics.

## Target state it writes

`apply_envelope_transactionally` opens one target transaction and, inside it, checks
`trellara.applied_transactions` for a prior apply, executes the planned row statements, then writes
`trellara.applied_transactions`, `trellara.flow_checkpoints`, and (in partitioned scale mode)
`trellara.partition_checkpoints`, and clears any prior `trellara.apply_quarantine` row for the same
transaction key. One commit makes all of it visible or none of it. The stream cursor is
acknowledged only after that commit returns.

An exact redelivery short-circuits: the duplicate is detected, quarantine is cleared, and the
transaction commits without re-executing any row statement.

DDL barrier acknowledgement is a separate step. `record_target_ddl_barrier_from_envelope` writes
`trellara.ddl_barriers` / `trellara.ddl_barrier_acks` through the `DdlBarrierStore` trait rather
than inside the row-apply transaction, because a barrier spans sinks the target transaction cannot
see. Post-DDL DML is released by `target_ddl_release_decision`, not by the apply transaction.

All of these tables are defined by `trellara-checkpoint`; this crate only writes them.

## Boundaries

This crate mutates target PostgreSQL. It does not capture source WAL, publish stream messages, define durable stream cursors, or own source acknowledgements. Durable state interfaces come from `trellara-checkpoint`; transaction semantics come from `trellara-protocol`.

All dynamic values must be bound parameters. Identifier and DDL construction must pass the crate's explicit safety validation.

## Key entry points

- `PostgresApplier`
- `ApplyWorker` and `BarrierAwareApplyWorker`
- `plan_change`, `plan_envelope`, and `SqlStatement`
- DDL plan, evidence, acknowledgement, and release APIs

## Development

```console
cargo test -p trellara-apply-postgres
cargo clippy -p trellara-apply-postgres --all-targets -- -D warnings
```

See [skills.md](skills.md) and the [workspace map](../README.md).
