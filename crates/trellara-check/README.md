# trellara-check

Standalone read-only PostgreSQL CDC source-safety diagnostic.

`trellara-check` is the small distribution surface for the first customer touch:

```console
trellara-check "$DATABASE_URL" --format html --output source-safety.html
```

It intentionally does not depend on `trellara-cli`. The binary reuses `trellara-pg-capture` for managed-Postgres-safe normal SQL connections, table discovery, logical slot inspection, WAL headroom projection, transaction ID wraparound evidence, xmin horizon pinners, subscription conflict counters, and the exact read-only query manifest. This crate owns only the no-config command shape, scoring, redaction, and text/JSON/HTML rendering.

## Boundaries

- Accept a single PostgreSQL connection string and optional output format/path.
- Run read-only SQL only; do not create publications, slots, checkpoints, schemas, or target state.
- Use normal PostgreSQL TLS connection handling so RDS, Aurora, Cloud SQL, Azure Database for PostgreSQL, and Neon use the same path as self-managed PostgreSQL.
- Keep recovery workflow, config generation, replication, apply, fleet, and pilot-package orchestration in `trellara-cli`.

## Output contract

One positional argument (the connection string), `--format text|json|html` (default `text`), and an
optional `--output` path. The summary is a `CheckStatus` of `healthy`, `degraded`, or `blocked`, a
`CheckGrade` of `A`-`F`, and a list of `CheckFactor` records carrying `code`, `severity`
(`warning` | `critical`), `points_lost`, `evidence`, and `recommendation`. The JSON shape and the
factor codes are the automation contract; the text and HTML renderings are for people.

Connection strings are redacted by `redact_database_url` before they reach any of the three
renderers.

## Key entry points

- `CheckArgs` and `CheckOutputFormat`
- `CheckSummary`, `CheckFactor`, `CheckGrade`, `CheckSeverity`, `CheckStatus`
- `render_check_summary`
- `redact_database_url`

## Verification

```console
cargo test -p trellara-check --lib
cargo run -p trellara-check -- --help
cargo build -p trellara-check --release
```
