# Trellara repository instructions

These instructions are the canonical starting point for coding agents and automated contributors
working in this repository. Human contributors should begin with [CONTRIBUTING.md](CONTRIBUTING.md).

## Mission and source of truth

Trellara is a source-safety and verified-replication layer for PostgreSQL. Correctness takes
priority over throughput or convenience: committed transaction boundaries must stay intact, source
progress advances only after durable publication, target mutation and checkpointing are atomic,
and convergence claims require reviewable evidence.

Executable contracts are authoritative. If prose conflicts with serialized types, configuration
validation, protocol fixtures, CLI help, SQL schemas, or tested constants, correct the prose or the
implementation as part of the same change.

## Read before changing code

1. Read the affected crate's `README.md` for its ownership boundary.
2. Read the affected crate's `skills.md` for package-specific engineering guardrails.
3. Read [docs/DESIGN.md](docs/DESIGN.md) before changing a correctness invariant, protocol field,
   acknowledgement boundary, checkpoint, DDL barrier, or recovery behavior.
4. Read [docs/ROADMAP.md](docs/ROADMAP.md) before changing a support or production-readiness claim.

More-local instructions take precedence when they are stricter. Do not duplicate domain logic in
the CLI when an owning crate already defines it.

## Repository map

- `crates/trellara-protocol`: versioned transaction and manifest contracts.
- `crates/trellara-pg-capture` and `crates/trellara-pg-extension`: external and native capture.
- `crates/trellara-relay`: durable publication and source acknowledgement sequencing.
- `crates/trellara-stream*`: transport abstraction plus local and Kafka adapters.
- `crates/trellara-apply-postgres`: idempotent PostgreSQL apply, quarantine, and target progress.
- `crates/trellara-checkpoint`: durable progress, barrier, snapshot, and evidence models.
- `crates/trellara-verify`: canonical comparison, drift detection, and reseed planning.
- `crates/trellara-lake` and `crates/trellara-iceberg`: epoch planning and Iceberg side effects.
- `crates/trellara-runtime`: lifecycle, readiness, metrics, and release-support vocabulary.
- `crates/trellara-check`: standalone read-only source diagnostic.
- `crates/trellara-cli`: operator composition and rendering.
- `crates/trellara-sim`: deterministic failure simulations and proof scenarios.

The complete ownership and feature map is in [crates/README.md](crates/README.md).

## Engineering rules

- Preserve Rust 1.97.1, edition 2021, and workspace-managed dependency versions.
- Prefer precise domain types, typed errors, checked arithmetic, explicit limits, and deterministic
  serialization. Avoid panics in operator- or input-facing paths.
- Treat protocol, config, CLI structured output, checkpoint schemas, metrics, and durable file
  formats as compatibility surfaces. Make migrations explicit.
- Fail closed on incomplete transactions, unknown versions, ordering conflicts, checksum mismatch,
  schema drift, and ambiguous durability.
- Never log or commit credentials, connection strings, unrestricted row values, tokens, private
  keys, or certificates. Keep production configuration on secret references.
- Do not perform destructive database recovery, cursor movement, reseed, unsafe DDL, or history
  rewrites unless the task explicitly authorizes it and the target is verified.
- Keep unrelated user changes intact. Inspect `git status` before editing and stage files by name.

## Verification

Run the narrow crate test while iterating, then the repository gate:

```console
cargo test -p <crate-name>
make ci
git diff --check
```

`make ci` runs formatting, Clippy, workspace tests, release builds, example validation, brokerless
quickstart checks, Compose validation, and correctness-report freshness checks.

Service-backed tests are opt-in. Use the documented environment and package command for PostgreSQL,
Kafka/Redpanda, Iceberg catalog, object-store, or pgrx tests. The common PostgreSQL apply path is:

```console
make dev-up
make integration-test
make dev-down
```

Do not run `cargo test --workspace --all-features`: PostgreSQL extension features `pg15`, `pg16`,
`pg17`, and `pg18` are mutually exclusive. When Kafka or lake code changes, also build the full CLI:

```console
cargo build -p trellara-cli --features full
```

## Documentation and generated evidence

- Keep the root README focused on evaluation, public guarantees, and current support claims.
- Update the owning crate's `README.md` and `skills.md` when its behavior or guardrails change.
- Update `docs/DESIGN.md` for architecture and invariant changes.
- Update `docs/ROADMAP.md` only when evidence or an explicit decision changes a readiness claim.
- Regenerate `docs/correctness-report.html` with `make correctness-report` when simulations or proof
  output change; confirm it with `make verify-correctness-report`.

Some tests include documentation text as fixtures. A prose-only change can therefore fail the build.

## Pull requests

Keep pull requests cohesive. Explain the user or operator impact, name the correctness boundaries
touched, list the exact checks run, and call out any service-backed checks not run. New public
behavior must include tests and matching documentation. Never weaken an invariant merely to make a
test pass.
