# Development standards: trellara-pg-extension

## Engineering goal

Keep the code running inside PostgreSQL small, bounded, observable, ABI-correct, and unable to acknowledge source WAL before the external relay proves durable publication.

## Rust and pgrx standards

- Preserve the workspace Rust 1.97.1 minimum for the feature-free contract core and PostgreSQL feature builds.
- Enable exactly one PostgreSQL feature (`pg15`, `pg16`, `pg17`, or `pg18`) per native build; never use `--all-features`.
- Keep PostgreSQL-independent contracts testable without `pg_config` or a running server.
- Isolate `unsafe` and raw `pg_sys` access behind small modules. Document memory ownership, process lifetime, locking, callback, and panic-safety invariants at every unsafe boundary.
- Never unwind through PostgreSQL C frames. Guard entry points with pgrx facilities and convert failures to controlled PostgreSQL errors.
- Avoid allocation, blocking I/O, logging storms, or remote calls in logical-decoding callbacks and critical sections.
- Use checked sizing and explicit upper bounds for shared memory, frames, batches, and metadata.

## Native data-plane guardrails

- `MAX_RUNTIME_QUEUE_FRAMES = 16` sizes the shared-memory ring at preload; `trellara.queue_capacity`
  (default 8) only chooses how many of those slots admit frames. Raising the GUC ceiling means
  resizing the static array, which is a postmaster-shared-memory change, not a config change.
- `MAX_LOGICAL_FRAME_BYTES = 64 KiB` caps one encoded frame. An oversized committed transaction is
  rejected with an explicit error; it is never truncated or split behind the operator's back.
- New GUCs register through `pg_guc/registration.rs` with an explicit `GucContext`. Anything that
  affects preload sizing or the target database is `Postmaster`; anything operationally tunable is
  `Sighup`. Secrets get `SUPERUSER_ONLY | NO_SHOW_ALL`, as `trellara.relay_secret` does.
- The queue is bounded. Full-queue behavior must apply backpressure or fail visibly; it must never drop committed frames silently.
- Publish only complete committed transaction boundaries. Abort and worker restart must not expose partial state.
- Preserve transaction order and verify frame identity/checksums at enqueue, drain, and feedback boundaries.
- Broker I/O remains outside PostgreSQL.
- Source feedback advances only when destination, checksum, LSN coverage, checkpoint, and durable-publish proofs all match.
- Shared memory and static workers require preload-time registration; SQL-only loading must not claim they are active.
- Fail closed for unsupported PostgreSQL majors and ABI mismatches.

## Testing expectations

- Keep pure contract tests for sizing, queue admission, ordering, lifecycle, hooks, worker supervision, and feedback proof validation.
- Add pgrx integration tests for every PostgreSQL major whose support claim is affected. The build/package matrix currently covers 15-18, while the shared release contract advertises native 17-18; do not broaden either claim without aligning runtime constants, CI, packaging, and live qualification evidence.
- Test queue exhaustion, oversized frames, worker restart, stale/conflicting feedback, crash-before-publish, and crash-after-publish-before-feedback.
- Do not set `data_plane_ready` based only on plans or simulated components.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-pg-extension --all-targets -- -D warnings
cargo test -p trellara-pg-extension
DOCS_RS=1 cargo check -p trellara-pg-extension --features pg17
DOCS_RS=1 cargo check -p trellara-pg-extension --features pg18
git diff --check
```

Run the matching `DOCS_RS=1 cargo check ... --features pg15|pg16|pg17|pg18` and `cargo pgrx test pg15|pg16|pg17|pg18` commands whenever a change affects that major. Never combine PostgreSQL features.
