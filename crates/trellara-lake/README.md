# trellara-lake

`trellara-lake` plans verified lakehouse consumption from the Trellara transaction stream. It models complete source epochs, raw CDC writes, replay deduplication, and derived current/SCD2 materializations.

## Responsibilities

- Define lake table and commit-plan configuration.
- Accumulate and validate complete source epochs and manifest boundaries.
- Produce deterministic epoch identities and recovery guidance.
- Plan raw CDC files, row intents, metadata, manifests, and commit operations.
- Encode planned raw CDC rows to Arrow/Parquet with stable field IDs and completion evidence.
- Encode the single-row `_trellara_epochs` completeness ledger to Arrow/Parquet once raw Iceberg snapshot evidence is available.
- Deduplicate replayed transactions without hiding conflicting payloads.
- Plan raw, current-state, and SCD2 materializations.
- Gate lake and derived-view visibility on completeness and required DDL acknowledgements.

## Raw CDC shape

Raw CDC is a fixed 29-column changelog, not a copy of the source table's columns. Row images travel
as canonical JSON in `payload_before_json` / `payload_after_json`; the other 27 columns carry source,
dataset, relation, transaction, ordering, LSN, operation, schema-fingerprint, DDL-barrier, envelope
checksum, manifest, partition, epoch, and ingestion lineage. `trellara-iceberg` owns the physical
schema and field IDs; this crate owns the row intents and the completeness metadata plan that fills
`_trellara_epochs`, `_trellara_epoch_sources`, `_trellara_epoch_tables`,
`_trellara_epoch_partitions`, `_trellara_quarantine`, and `_trellara_verification`.

Epoch state is `LakeCompletenessState` — `open`, `sealing`, `complete`, `complete_with_gaps`,
`quarantined`, `reseeding`, `failed_recoverable`. Per-source state is `LakeEpochSourceState` —
`complete`, `lagging`, `missing`, `quarantined`, `reseeding`. `complete_with_gaps` is consumable
only when the caller passes `LakeEpochConsumerOptions::accepting_complete_with_gaps`; the default is
strict and rejects it.

## Features

`parquet-writer` (off by default) pulls in `arrow-array`, `arrow-schema`, and `parquet` and enables
the raw CDC and `_trellara_epochs` encoders. Planning, validation, and the consumer gate build with
the default feature set and no Arrow dependency. `trellara-iceberg` enables `parquet-writer`
transitively; the `trellara-cli` `lake` feature turns it on for the CLI.

## Boundaries

This crate currently owns planning and validation, not a general object-store client, catalog runtime, query engine, or arbitrary transformation framework. Protocol transaction semantics remain in `trellara-protocol`; durable progress/evidence belongs in `trellara-checkpoint`; validated Iceberg catalog commits belong in `trellara-iceberg`.

No plan may claim a visible epoch when required partitions, manifests, commit markers, metadata, or DDL acknowledgements are incomplete.

## Key entry points

- `plan_commit` and `LakePlanConfig`
- Epoch accumulation, identity, summary, and recovery APIs
- `plan_raw_cdc_epoch_writes`
- Arrow/Parquet raw CDC and `_trellara_epochs` encoders
- Consumer and DDL acknowledgement gates

## Development

```console
cargo test -p trellara-lake
cargo clippy -p trellara-lake --all-targets -- -D warnings
```

See [skills.md](skills.md) and the [workspace map](../README.md).
