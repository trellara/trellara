# Trellara

Trellara is an open-source source-safety and verified replication layer for PostgreSQL. It checks whether a source is safe for change data capture (CDC), carries committed transactions across a durable boundary, applies them idempotently, and produces evidence that downstream state converged.

The repository implements two related paths:

1. verified PostgreSQL-to-PostgreSQL replication for operational copies and migrations; and
2. verified fleet fan-in into append-only Apache Iceberg raw CDC with explicit completeness epochs.

The external relay is the default path because it works with managed PostgreSQL. A native PostgreSQL extension is available for self-managed deployments that can install and preload an extension.

## Project status

Trellara is pre-1.0 and being prepared for design-partner qualification. The core correctness contracts, local durable transport, Kafka adapter, PostgreSQL capture/apply path, verification, failure simulation, native-extension data plane, and feature-gated Iceberg writer are implemented and tested in this repository.

The project does **not** yet claim broad production qualification. In particular, the cross-service live qualification harness, published multi-architecture release set, and customer-workload performance envelope remain roadmap work. See [the roadmap](docs/ROADMAP.md) for the evidence gates that must be cleared before those claims change.

## What Trellara guarantees

- **Read-only source safety first.** Inspect logical-replication settings, table identity, slot health, WAL retention, failover posture, and transaction-horizon risks before creating replication state.
- **Committed transaction boundaries.** Strict, strict-chunked, and partitioned modes carry manifests and commit markers that consumers validate before visibility.
- **Durability before source acknowledgement.** The relay advances source progress only after durable stream publication and durable source-checkpoint evidence agree.
- **Atomic target progress.** PostgreSQL mutation, deduplication, and target-checkpoint advancement occur in one target transaction; the stream cursor advances afterward.
- **Fail-closed recovery.** Incomplete barriers, schema drift, conflicting replay, checksum mismatch, and target failures become explicit quarantine or recovery states.
- **Verifiable convergence.** Canonical snapshots, row hashes, watermarks, proof bundles, and reseed plans make drift observable and repairable.
- **Destination neutrality.** The protocol and stream contracts are not coupled to one broker, catalog, object store, or hosted control plane.

The complete architecture and invariant definitions live in [docs/DESIGN.md](docs/DESIGN.md).

## Architecture

```text
PostgreSQL source
   |  pgoutput (external) or native extension
   v
capture -> relay -> durable stream (local log or Kafka/Redpanda)
                         |
             +-----------+-----------+
             |                       |
             v                       v
      PostgreSQL apply          lake epoch planner
      + checkpoint              + Parquet encoder
      + quarantine                    |
             |                        v
             v                 Iceberg REST/S3 writer
      convergence verify        + completeness ledger
```

The workspace contains 16 Rust crates. [crates/README.md](crates/README.md) maps responsibility and dependency boundaries crate by crate.

## Requirements

- Rust 1.97.1 through the pinned `rust-toolchain.toml`
- Docker and Docker Compose for the local PostgreSQL evaluation
- Optional: Kafka/Redpanda for the scale-out stream adapter
- Optional: PostgreSQL/pgrx toolchains for native-extension work

## Build

```sh
cargo build -p trellara-check --release
cargo build -p trellara-cli --release --no-default-features --features local-stream
cargo build -p trellara-cli --release --features full
```

The three commands produce the standalone diagnostic, the default brokerless CLI, and the full CLI with Kafka and lake features.

Tagged releases currently publish Linux x86_64 tarballs:

- `trellara-check-x86_64-unknown-linux-gnu.tar.gz`
- `trellara-x86_64-unknown-linux-gnu.tar.gz`
- `trellara-full-x86_64-unknown-linux-gnu.tar.gz`
- `SHA256SUMS`

Checksums are the only release provenance today: there is no signing, SBOM, or reproducible-build
attestation yet.

## Ten-minute local evaluation

The quickest complete path is:

```sh
make quickstart-local
```

That workflow starts disposable source and target PostgreSQL services, creates a local configuration, runs source checks and preflight, performs a bounded brokerless replication run with verification, renders status, and writes a reviewable pilot package.

The public command loop is:

```sh
cargo run -p trellara-cli -- init \
  --source-database-url postgresql://trellara:trellara@localhost:55432/trellara_source \
  --target-database-url postgresql://trellara:trellara@localhost:55433/trellara_target \
  --source-id local-source \
  --dataset-id retail-sales \
  --publication trellara_retail \
  --slot trellara_retail_slot \
  --table public.sales \
  --table public.sale_items \
  --table public.payments \
  --output trellara.yml \
  --evaluate \
  --force
cargo run -p trellara-cli -- check --config trellara.yml --format text
cargo run -p trellara-cli -- preflight --config trellara.yml
cargo run -p trellara-cli -- run --config trellara.yml --local --verify --format text \
  --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100
cargo run -p trellara-cli -- verify --config trellara.yml
cargo run -p trellara-cli -- status --config trellara.yml --view report --format text
```

Start the local services separately when running those commands by hand:

```sh
cargo run -p trellara-cli -- dev up
```

`dev` and other expert commands are intentionally hidden from the short public help surface. Run `cargo run -p trellara-cli -- --help` for the supported entry points and use the generated pilot package for advanced proof commands.

## Standalone source-safety check

`trellara-check` is a no-config, read-only first interaction with a PostgreSQL source. It does not create publications, slots, checkpoints, or target state.

```sh
./target/release/trellara-check "$DATABASE_URL" --format html --output source-safety.html
```

The same diagnostic is available through `trellara check`; `source-safety` remains a hidden compatibility alias.

## Public CLI map

| Command | Purpose |
| --- | --- |
| `init` | Write a validated version-2 flow configuration. |
| `check` | Inspect source CDC safety and render text, JSON, or HTML evidence. |
| `preflight` | Validate source capture and target schema contracts before mutation. |
| `run` | Compose bounded or continuous relay/apply work; `--local --verify` is the first-run path. |
| `verify` | Compare canonical source and target state and report drift. |
| `status` | Render health, proof, diagnostics, alerts, and recovery evidence. |
| `fleet` | Build and review multi-flow design-partner evidence. |
| `lake` | Plan, inspect, write, and verify lakehouse epochs and derived templates. |
| `config` | Migrate or redact configuration safely. |

Commands for schema barriers, stream repair, quarantine, snapshots, simulation, and evidence collection remain available as expert/compatibility surfaces but are not the primary onboarding path.

## Transport and consistency modes

The default local adapter is an append-only `TLG2` segment log with CRC32-framed records, a sidecar index, per-consumer cursors, monotonic offsets, torn-tail recovery, and `fsync` durability by default. Kafka/Redpanda is the production scale-out adapter and validates broker topology, replication, ISR quorum, TLS/authentication, `acks=all`, producer idempotence, and consumer progress rules before treating a publish as durable.

Boundary behaviour is chosen per flow, and the configuration vocabulary is narrower than the wire
vocabulary. `dataset.mode` takes exactly two values:

- `strict_transaction_order` — one complete transaction envelope becomes visible as one unit. Adding
  `dataset.strict_chunking.max_changes_per_chunk` keeps this mode but splits every transaction into
  ordered chunks followed by a manifest and a manifest-bound commit marker, so a large transaction
  stays hidden until the complete set is proven. There is no separate `strict_chunked` mode value.
- `partitioned_scale_mode` — partition-local chunks may scale independently, but global visibility
  waits for the manifest, commit marker, and the partition watermark barrier across the complete
  configured partition set.

On the wire, manifests carry `ManifestBoundaryMode`: `strict_chunked_transaction_order` or
`partitioned_scale_mode`. A plain strict transaction carries no manifest at all.

## Lakehouse path

The lake path writes an append-only raw CDC changelog with a fixed 29-column schema. It is not a
projection of the source table: row images travel as canonical JSON in `payload_before_json` and
`payload_after_json`, and 27 flat columns carry source, dataset, relation, transaction, ordering,
LSN, operation, schema-fingerprint, DDL-barrier, checksum, manifest, partition, epoch, and ingestion
lineage. Raw tables are partitioned by `identity(epoch_id), identity(source_bucket)`. Six
unpartitioned `_trellara_*` support tables carry the completeness ledger, and `_trellara_epochs` —
the single row consumers gate on — is committed last. The feature-gated production writer uses an Iceberg REST catalog and S3-compatible object storage, records deterministic intents and receipts, validates uploaded objects by checksum/read-back evidence, and reconciles ambiguous commits.

Current-state and SCD2 outputs are rendered as epoch-gated Spark templates. Trellara does not currently provide a native equality-delete/current-state Iceberg writer. Cross-table visibility is a Trellara release-ordering contract; Apache Iceberg commits remain atomic per table.

## Compatibility and release claims

| Surface | Current claim |
| --- | --- |
| External relay contract | PostgreSQL 16, 17, and 18 on Linux |
| Native extension release contract | PostgreSQL 17 and 18 on Linux |
| Native extension CI/package matrix | PostgreSQL 15, 16, 17, and 18 |
| Runtime architecture vocabulary | `amd64`, `arm64` |
| Currently published CLI artifacts | Linux x86_64 tarballs |

The native build matrix and advertised release contract intentionally remain distinct here: CI coverage is not automatically a support promise. Aligning the two with live qualification evidence is a roadmap gate.

## Configuration and secrets

The current configuration schema is version 2. A flow config has `config_version`, an `environment`
of `development` or `production`, and four blocks — `source`, `dataset`, `stream`, and an optional
`target`. Unknown fields are rejected rather than ignored.

Development flows may use the local stream and inline database URLs. Production validation requires
secret references (`{source: environment_variable, name: ...}` or `{source: file, path: ...}`) and
the Kafka production profile rather than embedding credentials in configuration. Working examples of
both live in [`examples/retail-fleet/`](examples/retail-fleet). Use:

```sh
cargo run -p trellara-cli -- config migrate --config trellara.yml
cargo run -p trellara-cli -- config redact --config trellara.yml
```

## Verification

Run focused tests while developing and the workspace gates before landing:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

`make ci` runs the same gate CI does, plus the three release builds, the example-config validation,
the no-broker quickstart checks, the compose config check, and the correctness-report freshness
checks.

Service-backed tests are opt-in because they require PostgreSQL, Kafka/Redpanda, Docker, an Iceberg catalog, or pgrx toolchains. CI runs no PostgreSQL service, so nothing in the default pipeline exercises a live server. The generated correctness report and failure matrix are preserved at [docs/correctness-report.html](docs/correctness-report.html).

## Documentation

- [Design](docs/DESIGN.md) — authoritative architecture, invariants, implementation status, and operating contracts.
- [Roadmap](docs/ROADMAP.md) — domain analysis, positioning, risks, evidence gates, and phased execution plan.
- [Workspace guide](crates/README.md) — crate ownership and dependency map.
- Each crate's `README.md` and `skills.md` — package-specific behavior and contributor guardrails.

If prose conflicts with serialized types, configuration validation, protocol fixtures, CLI help, or tested runtime constants, the executable contract wins and the prose should be corrected.

## License

See [LICENSE](LICENSE).
