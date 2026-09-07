# trellara-cli

`trellara-cli` is Trellara's composition root and operator-facing product surface. It parses configuration and commands, invokes domain packages, and renders actionable diagnostics, evidence, reports, and recovery guidance through the `trellara` binary.

## Responsibilities

- Define command-line arguments, configuration files, validation, and explain output.
- Run bootstrap, preflight, source-safety checks, contract, supervised relay/apply services, verification, and bounded local workflows.
- Expose stream inspection/seek, snapshot, DDL barrier, quarantine, repair, and transaction-inspection commands.
- Run deterministic chaos, pilot, fleet, lake, and evidence/reporting workflows.
- Render stable human-readable and machine-readable output.
- Compose PostgreSQL, stream, checkpoint, simulation, lake, and verification packages.

## Visible command surface

`trellara --help` shows the public replication loop plus `fleet`, `lake`, and the safe `config`
migration/redaction surface. The public loop is `init -> check -> preflight -> run -> verify -> status`.

`check` is the primary source-safety diagnostic; `source-safety` remains a hidden compatibility
alias dispatching to the same handler. Every other subcommand listed under Responsibilities —
chaos, pilot, dev, contract, schema, stream, snapshot, quarantine, repair, transaction inspection,
partition-* — still works but is deliberately hidden from `--help` as an operator/diagnostic
surface rather than supported public CLI. Add `#[command(hide = true)]` to anything new that is
not part of the public loop.

## Cargo features

| Feature | Default | Pulls in | Enables |
| --- | :---: | --- | --- |
| `local-stream` | yes | `trellara-stream-local` | The brokerless `TLG2` transport and `stream *-local` commands. |
| `kafka` | no | `trellara-relay/kafka`, `trellara-stream-kafka/runtime` | Kafka/Redpanda publish and consume. |
| `lake` | no | `trellara-iceberg`, `trellara-lake/parquet-writer` | Parquet encoding and Iceberg catalog writes behind `lake`. |
| `full` | no | all three | The release `trellara-full` binary. |

`cargo test -p trellara-cli` exercises the default feature set only. A change that touches the
Kafka or lake command paths needs `--features kafka`, `--features lake`, or `--features full` as
well, which is what CI builds before release.

## Configuration surface

`TrellaraConfig` is `#[serde(deny_unknown_fields)]` with `config_version`, `environment`
(`development` | `production`), and four blocks: `source`, `dataset`, `stream`, and an optional
`target`. `CURRENT_CONFIG_VERSION` is `2`; `config migrate` upgrades known older versions and
rejects unknown future ones, and `config redact` renders a shareable copy without secrets.
`dataset.mode` is `strict_transaction_order` or `partitioned_scale_mode` — strict chunking is
`dataset.strict_chunking.max_changes_per_chunk` under the strict mode, not a third mode value.
There is no `lake` block; lake planning takes its table set from the dataset.

## Boundaries

The CLI should coordinate public package APIs rather than become a second domain layer. Reusable protocol, capture, relay, checkpoint, apply, verification, stream, and lake invariants belong in their owning crates.

Command execution must validate before mutation, make destructive/irreversible effects explicit, and avoid exposing credentials or row data in default output.

## Structure

- `args*` — command and option definitions
- `config*` — configuration models, validation, conversion, and explanation
- runtime modules — bootstrap, producer/consumer, snapshot, relay/apply, and verification composition
- service modules — continuous polling, reconnect/backoff, signal-driven drain/reload, and runtime health endpoints
- operator modules — status, diagnostics, repair, DDL, quarantine, pilot, fleet, lake, and `source_safety/` workflows
- evidence/report modules — structured artifacts and human/machine rendering
- `module_manifest*` — transitional wiring while flat CLI modules are moved into cohesive directories; treat manifests as generated wiring, not a second ownership model

## Development

```console
cargo test -p trellara-cli
cargo clippy -p trellara-cli --all-targets -- -D warnings
cargo run -p trellara-cli -- --help
```

See [skills.md](skills.md) and the [workspace map](../README.md).

## Module migration

Flat modules move into real directories one filename-prefix cluster at a time:

```console
./scripts/migrate-cli-cluster.sh <prefix>            # dry run (default)
./scripts/migrate-cli-cluster.sh <prefix> --apply
cargo test --workspace
```

Run the focused CLI checks after each cluster and the full workspace suite before landing. The
script refuses to run on an uncommitted `trellara-cli` tree, so review and checkpoint any
in-progress work before using `--apply`.

A cluster root is always `<prefix>/mod.rs`, never `<prefix>.rs`: a `#[path]`-loaded module
resolves its children relative to its file's directory without appending the module name, so a
bare `<prefix>.rs` would search `src/` instead of `src/<prefix>/`. Only the cluster root needs
`#[path]`; everything below it resolves as ordinary Rust.
