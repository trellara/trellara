# Development standards: trellara-checkpoint

## Engineering goal

Persist progress and correctness evidence monotonically so replay, recovery, verification, and operators can distinguish proven durability from intent.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Use typed records and errors; validate at every in-memory and PostgreSQL store boundary.
- Keep canonicalization explicit. Persist canonical transaction IDs, LSNs, partition IDs, digests, and timestamps.
- Avoid panics, lossy casts, implicit defaults, and read-modify-write sequences without transactional protection.
- Keep SQL centralized, parameterized, and exercised by schema/row conversion tests.
- Ensure in-memory and PostgreSQL implementations enforce the same observable invariants.
- Keep state transitions small and reviewable; complex validation belongs in pure helper modules.

## State guardrails

- Checkpoints and watermarks are monotonic unless a named administrative recovery operation explicitly says otherwise.
- Exact duplicate writes may be idempotent; conflicting duplicates must fail closed.
- Do not record an acknowledgement, release, handoff, or validation fact before its required evidence is durable.
- Snapshot and DDL state machines reject invalid transitions and stale/mismatched identities.
- Multi-record state changes that represent one correctness boundary must use one database transaction.
- Schema changes must be backward-aware and safe for partially upgraded callers.

## Testing expectations

- Run each invariant against both in-memory and PostgreSQL row/SQL paths where applicable.
- Add transition-table tests for snapshots and DDL barriers.
- Use property tests for monotonicity, ordering, lag, and duplicate behavior.
- Test malformed persisted rows, stale evidence, digest mismatch, and concurrent/replayed writes.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-checkpoint --all-targets -- -D warnings
cargo test -p trellara-checkpoint
git diff --check
```
