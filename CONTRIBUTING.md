# Contributing to Trellara

Thank you for helping make PostgreSQL replication safer and easier to verify. Trellara is pre-1.0,
so focused bug reports, reproducible failure cases, documentation corrections, and tests for sharp
edges are especially valuable.

By participating, you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md). Report suspected
security vulnerabilities through the private process in [SECURITY.md](SECURITY.md), not through a
public issue.

## Before opening a change

- Search existing issues and pull requests for related work.
- For a small fix, documentation correction, or added test, a pull request is welcome directly.
- For a protocol change, new command, compatibility break, or large design change, open an issue
  first so the correctness contract and migration path can be agreed before implementation.
- Keep changes scoped to one concern. Avoid combining cleanup with behavior changes.

## Development setup

Install:

- Rust 1.97.1 through the repository's `rust-toolchain.toml`;
- native build tools, CMake, `pkg-config`, and libcurl development headers;
- Docker with Docker Compose for the local PostgreSQL evaluation; and
- optional Kafka/Redpanda, Iceberg, object-store, or pgrx tooling for the integration you touch.

Clone and run the fast workspace tests:

```console
git clone https://github.com/trellara/trellara.git
cd trellara
cargo test --workspace
```

The workspace contains 16 crates. Use [crates/README.md](crates/README.md) to find the package that
owns your change, then read that package's `README.md` and `skills.md` before editing it.

## Development workflow

1. Create a branch from current `main`.
2. Add or update tests with the behavior change.
3. Run the narrow package checks while iterating:

   ```console
   cargo fmt --all -- --check
   cargo clippy -p <crate-name> --all-targets -- -D warnings
   cargo test -p <crate-name>
   ```

4. Run the complete repository gate before opening a pull request:

   ```console
   make ci
   git diff --check
   ```

`make ci` checks formatting, Clippy, all default workspace tests, release builds, example
configuration, the brokerless quickstart, Docker Compose configuration, and generated correctness
evidence.

Do not use `cargo test --workspace --all-features`. The `pg15`, `pg16`, `pg17`, and `pg18` extension
features are mutually exclusive. If you change Kafka or lake code, also run:

```console
cargo build -p trellara-cli --features full
```

## Service-backed tests

Default workspace tests do not require an external service. Tests that need PostgreSQL,
Kafka/Redpanda, an Iceberg catalog, object storage, or a pgrx toolchain are opt-in and documented by
their owning package.

For the common PostgreSQL apply integration suite:

```console
make dev-up
make integration-test
make dev-down
```

State which service-backed tests you ran in the pull request. If you could not run one, say so; do
not imply coverage that was not executed.

## Correctness and compatibility

Changes to these surfaces require an explicit compatibility review and matching tests:

- serialized protocol envelopes, manifests, checksums, or routing keys;
- source acknowledgement and durable-publication ordering;
- target mutation, deduplication, quarantine, or checkpoint transactions;
- configuration versions, CLI structured output, or exit behavior;
- durable local-log formats, database schemas, and metrics; and
- DDL barriers, lake completeness epochs, verification, and reseed evidence.

Trellara fails closed. Do not weaken validation or accept partial evidence solely to make a test pass.

## Documentation

Update documentation in the same pull request when behavior changes:

- root `README.md` for public evaluation, guarantees, or support claims;
- the owning crate's `README.md` and `skills.md` for package behavior and contributor guardrails;
- `docs/DESIGN.md` for architecture and invariant changes; and
- `docs/ROADMAP.md` only when new evidence or a deliberate decision changes readiness.

If simulation behavior or proof output changes, regenerate and verify the tracked report:

```console
make correctness-report
make verify-correctness-report
```

Some tests include documentation text as fixtures, so prose changes can fail the build.

## Pull request checklist

A useful pull request explains:

- the operator or user problem;
- what changed and which correctness boundary it touches;
- how the change fails safely;
- the exact checks and service-backed tests run;
- documentation or generated evidence updated; and
- any compatibility, migration, performance, or operational consequence.

Maintainers may ask for a smaller change, additional fixtures, or a migration plan when a patch
crosses a durable boundary.
