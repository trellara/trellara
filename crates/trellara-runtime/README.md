# trellara-runtime

`trellara-runtime` is the shared contract package for continuously running Trellara services. It freezes lifecycle states, readiness semantics, metric names, and release support targets consumed today by the `trellara-cli` service layer and intended for any future standalone runtime binary.

## Responsibilities

- Define versioned relay, applier, and Iceberg-writer lifecycle states.
- Validate which states may accept new work.
- Define stable runtime metric names and required low-cardinality labels.
- Define the supported PostgreSQL-major, operating-system, architecture, component, and package vocabulary.

## The contracts, concretely

`RUNTIME_CONTRACT_VERSION` is `1`. `RuntimeHealth` carries the contract version, service, source and
dataset identity, phase, readiness, `accepting_work`, `pending_work`, and an optional `reason_code`
restricted to lowercase ASCII, digits, and underscores.

| Enum | Values |
| --- | --- |
| `RuntimeService` | `relay`, `applier`, `iceberg_writer` |
| `RuntimePhase` | `starting`, `running`, `draining`, `stopped`, `failed` |
| `RuntimeReadiness` | `not_ready`, `ready`, `degraded`, `backpressured`, `blocked` |

`RuntimeHealth::validate` is the whole point of the crate: only `running` may accept work, and only
under `ready` or `degraded`; `backpressured` and `blocked` are running-but-not-accepting;
`starting`, `draining`, and `stopped` must be `not_ready`; `failed` must be `blocked`. Degraded,
backpressured, and blocked readiness all require a `reason_code`.

The seven contract-locked metric names, each carrying exactly the labels
`RUNTIME_METRIC_LABELS = ["service", "source_id", "dataset_id"]`:

- `trellara_runtime_live`
- `trellara_runtime_ready`
- `trellara_runtime_pending_work`
- `trellara_runtime_restarts_total`
- `trellara_runtime_failures_total`
- `trellara_runtime_last_success_unixtime`
- `trellara_runtime_last_durable_lsn_bytes`

`RELEASE_CONTRACT_VERSION` is `1`, with `SUPPORTED_EXTERNAL_POSTGRES_MAJORS = [16, 17, 18]` and
`SUPPORTED_NATIVE_POSTGRES_MAJORS = [17, 18]`.

## Boundaries

This package contains contracts only. It does not start tasks, handle signals, retry I/O, expose an HTTP server, connect to PostgreSQL or Kafka, write Iceberg files, or publish release artifacts. Runtime implementations must consume these types instead of inventing their own lifecycle vocabulary.

## Current advertised release contract

- External PostgreSQL relay: PostgreSQL 16, 17, and 18.
- Native PostgreSQL extension: PostgreSQL 17 and 18.
- Operating system: Linux.
- Architectures: `amd64` and `arm64` contract vocabulary.
- Components: CLI, relay, applier, and native extension.
- Package formats: `tar.gz`, Debian, RPM, and OCI vocabulary.

These values describe the compatibility contract, not an inventory of artifacts currently published by CI. Tagged CLI releases currently produce Linux x86_64 tarballs; native-extension CI separately builds/packages PostgreSQL 15-18. Keep those implementation matrices visibly distinct until release automation and the contract agree.

## Development

```console
cargo test -p trellara-runtime
cargo clippy -p trellara-runtime --all-targets -- -D warnings
```

See [skills.md](skills.md), the [consolidated design](../../docs/DESIGN.md#runtime-operations-and-release-contracts), and the [workspace map](../README.md).
