# Trellara Rust workspace

The `crates/` directory contains Trellara's production domain libraries, infrastructure adapters, operator CLI, and deterministic simulation tools. The workspace is organized around explicit correctness boundaries: PostgreSQL capture produces versioned protocol transactions, a relay publishes them durably, targets apply them idempotently, and verification proves convergence.

Each package contains:

- `README.md` — package responsibilities, boundaries, and common commands;
- `skills.md` — contributor standards and package-specific engineering guardrails.

## Architecture

```text
PostgreSQL source
  |-- client-side pgoutput ----------> trellara-pg-capture --|
  `-- native extension --------------> trellara-pg-extension |
                                                           v
                                                   trellara-relay
                                                           |
                                     +---------------------+---------------------+
                                     |                                           |
                              trellara-stream                           trellara-checkpoint
                                     |
                         +-----------+-----------+
                         |                       |
               trellara-stream-local   trellara-stream-kafka
                         |                       |
                         +-----------+-----------+
                                     |
                          +----------+----------+
                          |                     |
              trellara-apply-postgres      trellara-lake
                                                  |
                                          trellara-iceberg
                          |
                   trellara-verify

trellara-protocol is the shared domain contract.
trellara-runtime is the shared service lifecycle and operations contract.
trellara-check is the standalone read-only source-safety diagnostic.
trellara-cli composes the production packages.
trellara-sim exercises the boundaries deterministically.
```

Arrows show the primary runtime flow, not every Cargo dependency. Keep dependency direction toward stable domain contracts and traits. In particular, infrastructure adapters should depend on `trellara-stream`, while `trellara-stream` must not depend on a specific broker.

## Package map

| Package | Layer | Responsibility |
| --- | --- | --- |
| [`trellara-protocol`](trellara-protocol/README.md) | Domain core | Versioned transaction envelopes, DDL events, partition manifests, ordering, validation, checksums, and idempotency keys. |
| [`trellara-pg-capture`](trellara-pg-capture/README.md) | Source adapter | PostgreSQL logical replication, pgoutput decoding, source inspection, relation metadata, and transaction assembly. |
| [`trellara-pg-extension`](trellara-pg-extension/README.md) | Native source adapter | PostgreSQL-major-specific native capture, shared-memory handoff, worker lifecycle, and durable-feedback readiness. CI builds and exercises PostgreSQL 15-18; the shared release contract currently advertises 17-18. |
| [`trellara-stream`](trellara-stream/README.md) | Port/domain boundary | Transport-neutral messages, topic/key/header rules, and publisher/consumer traits. |
| [`trellara-stream-local`](trellara-stream-local/README.md) | Infrastructure adapter | Brokerless durable segment log, indexes, cursors, replay, inspection, and barrier reconstruction. |
| [`trellara-stream-kafka`](trellara-stream-kafka/README.md) | Infrastructure adapter | Kafka/Redpanda publisher and consumer implementations. |
| [`trellara-relay`](trellara-relay/README.md) | Orchestration | Capture-to-stream publication, relay modes, durable publish proofs, source checkpoints, and source acknowledgements. |
| [`trellara-runtime`](trellara-runtime/README.md) | Operations contract | Versioned service lifecycle, readiness, metric names, and the currently advertised Linux/architecture/PostgreSQL release targets. |
| [`trellara-apply-postgres`](trellara-apply-postgres/README.md) | Target adapter | PostgreSQL apply planning/execution, deduplication, target checkpoints, quarantine, and DDL/partition barriers. |
| [`trellara-checkpoint`](trellara-checkpoint/README.md) | State/evidence | Source and target progress, snapshot state, DDL acknowledgements, partition watermarks, quarantine, and validation evidence. |
| [`trellara-verify`](trellara-verify/README.md) | Verification | Canonical source/target snapshots, checksums, drift comparison, inspection, and reseed operations. |
| [`trellara-lake`](trellara-lake/README.md) | Downstream planner | Epoch completeness, raw CDC writes, current/SCD2 materialization plans, replay deduplication, and DDL gates. |
| [`trellara-iceberg`](trellara-iceberg/README.md) | Infrastructure adapter | Validated Parquet-file commits, deterministic Iceberg snapshot lineage, idempotent fast append, and cross-table receipt gating. |
| [`trellara-sim`](trellara-sim/README.md) | Test support | Seeded failure simulations for relay, apply, snapshot, strict chunking, fleet fan-in, and qualification/observability (5 suites, 28 default scenarios). |
| [`trellara-check`](trellara-check/README.md) | Standalone diagnostic | No-config PostgreSQL source-safety binary, read-only evidence rendering, and shareable reports. |
| [`trellara-cli`](trellara-cli/README.md) | Composition/operator surface | Commands, configuration, preflight, runtime orchestration, diagnostics, evidence, and reports. |

## Dependency rules

1. Put cross-package wire and transaction semantics in `trellara-protocol`.
2. Put transport abstractions in `trellara-stream`; keep local and Kafka details in their adapter crates.
3. Keep source decoding in capture packages and durable publication/acknowledgement orchestration in `trellara-relay`.
4. Keep durable progress and evidence models in `trellara-checkpoint`; callers own workflow decisions.
5. Keep target mutation in target packages. Keep lake planning in `trellara-lake` and Iceberg catalog side effects in `trellara-iceberg`.
6. Avoid dependency cycles and reverse dependencies from domain crates into adapters or the CLI.
7. Keep `trellara-check` independent of `trellara-cli`; it may reuse capture/evidence crates but must remain a small one-download diagnostic.
8. Runtime services consume lifecycle and metric vocabulary from `trellara-runtime`; adapters keep their production configuration contracts in the owning package.

## Where changes belong

| Change | Primary package |
| --- | --- |
| Envelope field, ordering rule, manifest invariant | `trellara-protocol` |
| pgoutput parsing or source metadata | `trellara-pg-capture` |
| Native PostgreSQL hook, shared memory, or worker contract | `trellara-pg-extension` |
| Publish/consume trait or common stream metadata | `trellara-stream` |
| Local disk or Kafka behavior | Corresponding stream adapter |
| Durable-publish/source-ACK sequencing | `trellara-relay` |
| Service lifecycle, readiness, runtime metrics, release targets | `trellara-runtime` |
| Target SQL, dedup, quarantine, or apply barrier | `trellara-apply-postgres` |
| Checkpoint schema, snapshot state, or evidence | `trellara-checkpoint` |
| Row comparison, hashing, inspection, or reseed | `trellara-verify` |
| Lake epoch or materialization planning | `trellara-lake` |
| Iceberg file validation, catalog append, snapshot lineage, or commit receipt | `trellara-iceberg` |
| Failure scenario/model | `trellara-sim` |
| No-config one-shot source-safety diagnostic | `trellara-check` |
| Command parsing, operator workflow, or rendering | `trellara-cli` |

## Cargo features

Most packages build with no features. Five have optional surfaces, and a change to any of them
needs the matching build, because the default `cargo test --workspace` does not compile them:

| Package | Feature | Default | Effect |
| --- | --- | :---: | --- |
| `trellara-cli` | `local-stream` | yes | Brokerless `TLG2` transport and `stream *-local` commands. |
| `trellara-cli` | `kafka` | no | Kafka publish/consume through `trellara-relay/kafka`. |
| `trellara-cli` | `lake` | no | `trellara-iceberg` plus Parquet encoding. |
| `trellara-cli` | `full` | no | All three; the released `trellara-full` binary. |
| `trellara-relay` | `kafka` | no | Kafka native-relay durability mode. |
| `trellara-stream-kafka` | `runtime` | yes | The rdkafka client; off leaves only the production contract types. |
| `trellara-lake` | `parquet-writer` | no | Arrow/Parquet encoders. |
| `trellara-iceberg` | `iceberg-rust` / `production-writer` / `live-catalog-tests` | no | Catalog commits, then REST/S3/OpenDAL orchestration, then opt-in live tests. |
| `trellara-pg-extension` | `pg15` / `pg16` / `pg17` / `pg18` | no | Exactly one per build; never `--all-features`. |

`trellara-pg-extension` is the sharp edge: PostgreSQL extension binaries are major-version
specific, so `--all-features` on the workspace does not compile.

## Workspace verification

All packages inherit the workspace Rust 1.97.1 minimum. The repository, CI, and Docker builder are pinned to that same patch release.

Run focused package tests while iterating, then the workspace gates before landing:

```console
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

That is the same order CI runs, plus the release builds it does afterwards:

```console
cargo build --release -p trellara-check
cargo build --release -p trellara-cli --no-default-features --features local-stream
cargo build --release -p trellara-cli --features full
```

`make ci` runs the full gate, including the example-config validation, the no-broker quickstart
checks, the compose config check, and the correctness-report freshness checks.

`docs/DESIGN.md` is a build dependency: `trellara-protocol`, `trellara-stream-local`, and
`trellara-checkpoint` each `include_str!` it and assert exact phrases, and several
`trellara-cli` freshness gates read it and `README.md` at runtime. Editing prose in those documents
can fail tests — run `make quickstart-proof-check` as well as `cargo test --workspace` after a docs
change.

Some integration tests require PostgreSQL, Kafka/Redpanda, Docker, or a configured pgrx toolchain. Package READMEs identify those requirements. Do not make ordinary unit tests depend on external services. CI does not currently run a PostgreSQL service, so anything that needs a live server is opt-in and unverified by the default pipeline.
