# Development standards: trellara-check

## Engineering goal

Give an operator a trustworthy, zero-config first read on whether a PostgreSQL source is safe for CDC, without depending on Trellara's replication runtime and without writing anything to the source. This is the first customer touch: it must run against an unfamiliar production database and be obviously safe to run.

## Rust standards

- Preserve the workspace MSRV and edition; use workspace-managed dependency versions.
- Keep argument parsing, inspection, scoring, and rendering in separate modules.
- Use typed factors, severities, grades, and statuses; avoid ad hoc strings for check logic.
- Return typed errors with actionable context; do not panic on connection or query failure.
- Keep output deterministic so reports can be diffed across runs.
- Keep the dependency closure small enough to cross-compile and ship as a standalone binary.

## Check guardrails

- Preserve the one-argument happy path: `trellara-check <DATABASE_URL>`.
- Do not add dependencies on `trellara-cli`, stream adapters, target apply, or runtime orchestration.
- Put reusable PostgreSQL inspection in `trellara-pg-capture`; keep this crate focused on the command, scoring, redaction, and report rendering.
- Never add write queries. Any query exposed by this crate must also appear in the shared read-only query manifest.
- Redact credentials before rendering text, JSON, or HTML.
- Use normal PostgreSQL TLS connection handling so RDS, Aurora, Cloud SQL, Azure Database for PostgreSQL, and Neon take the same path as self-managed PostgreSQL.

## Testing expectations

- Test factor scoring, redaction, and each render format (text, JSON, HTML) independently.
- Cover degraded and missing-evidence cases (no failover slot, no subscriptions, no logical slots) alongside the healthy case.
- Assert that no query outside the shared read-only manifest is issued.
- Keep default unit tests free of a live PostgreSQL dependency; mark real-connection flows as explicit integration tests.
- Add a redaction test for every new output surface before it ships.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-check --all-targets -- -D warnings
cargo test -p trellara-check --lib
cargo run -p trellara-check -- --help
git diff --check
```
