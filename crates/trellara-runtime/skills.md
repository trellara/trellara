# Development standards: trellara-runtime

## Engineering goal

Keep production lifecycle, readiness, observability, and release contracts small, versioned, deterministic, and shared by every runtime implementation.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Use typed enums and validated records instead of stringly typed runtime states.
- Keep this package free of network, filesystem, signal, task-runtime, and process side effects.
- Treat serialized names, metric names, labels, supported versions, and state meanings as compatibility contracts.
- Treat release matrices as advertised compatibility promises. Keep them aligned with packaging workflows, published artifacts, and live qualification evidence; document any temporary mismatch explicitly.
- Avoid secret-bearing fields and high-cardinality metric labels.
- Reject invalid state combinations explicitly; do not repair or default operator evidence silently.

## Contract guardrails

- A state combination is valid or it is an error. Never let a caller repair an invalid
  phase/readiness pair by defaulting a field.
- Only `running` accepts work, and only under `ready` or `degraded`. If a new readiness value is
  proposed, decide its `accepting_work` answer first — that answer is the contract.
- `degraded`, `backpressured`, and `blocked` require a `reason_code`. Reason codes stay lowercase
  ASCII, digits, and underscores so they are safe as a metric label value.
- Metric labels stay at `service`, `source_id`, `dataset_id`. Do not add a label that can take a
  per-transaction, per-table, or per-partition value.
- Renaming a metric, a serialized enum value, or a lifecycle state is a breaking change. Bump
  `RUNTIME_CONTRACT_VERSION` and add a fixture rather than editing in place.
- `SUPPORTED_EXTERNAL_POSTGRES_MAJORS` and `SUPPORTED_NATIVE_POSTGRES_MAJORS` are promises, not
  wishes. Widening either one requires a matching automated lane; narrowing one requires a
  deprecation note in the design document.

## Testing expectations

- Test every lifecycle/readiness combination that may accept or reject work.
- Test contract-version rejection and blank identities.
- Assert exact metric names, label sets, and supported release targets.
- Add compatibility fixtures before changing serialized names or meanings.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-runtime --all-targets -- -D warnings
cargo test -p trellara-runtime
git diff --check
```
