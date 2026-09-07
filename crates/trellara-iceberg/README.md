# trellara-iceberg

`trellara-iceberg` turns a validated Trellara raw-CDC epoch write plan and completed Parquet-file evidence into idempotent Apache Iceberg table appends.

## Responsibilities

- Map Trellara lake tables to Iceberg table identifiers.
- Validate that completed Parquet files exactly match the planned object keys, row counts, relations, source buckets, and checksum rollups.
- Derive deterministic epoch, table-commit, and commit UUID identities.
- Attach Trellara lineage and completeness evidence to Iceberg snapshot properties.
- Plan a stable raw-CDC Iceberg schema and an epoch/source-bucket partition contract, including DDL barrier evidence.
- Plan the fan-in L1 table set: append-only raw changelog tables plus the queryable `_trellara_epochs` completeness table.
- Validate and plan the `_trellara_epochs` append only after raw changelog receipts prove every table reached the same epoch commit.
- Detect exact replay and return an `AlreadyCommitted` receipt instead of appending duplicate files.
- Keep epoch metadata invisible until matching receipts exist for every planned table append.
- Define the versioned production writer target for REST catalogs, S3-compatible stores, partitioned append-only raw CDC, durable upload proof, intent/receipt ordering, and DDL-gated schema evolution.
- Validate REST catalog and S3-compatible object-store configuration contracts.
- Plan raw CDC table provisioning and DDL-gated schema/partition evolution.
- Build the Apache Iceberg REST catalog with an OpenDAL S3 file I/O factory.
- Encode and conditionally upload immutable raw and metadata Parquet objects.
- Provision or additively evolve tables only with exact DDL acknowledgement evidence.
- Commit source, table, partition, quarantine, verification, and completeness metadata.
- Reconcile ambiguous object writes and catalog commits without duplicate appends.
- Plan small-file compaction/snapshot cleanup and safely delete proven orphan objects.

## Boundaries

`trellara-lake` owns pure epoch and raw-CDC planning plus raw CDC Arrow/Parquet encoding. This crate owns object upload, table provisioning, Iceberg catalog commits, metadata publication, and maintenance planning. It does not capture PostgreSQL changes or advance source acknowledgements.

The runtime supports Parquet fast-appends to unpartitioned metadata tables and raw tables whose default partition spec is exactly Trellara's `identity(epoch_id), identity(source_bucket)` contract. `write_production_iceberg_epoch` is the full orchestration entry point. It provisions every table, uploads immutable objects, persists checkpoint intents, commits raw and supporting metadata snapshots, and publishes `_trellara_epochs` last.

Completed object evidence includes a lowercase SHA-256 content digest and an optional immutable object version in addition to URI, byte size, row count, relation, source bucket, and logical checksum rollup. The production writer derives this evidence from the completed upload rather than trusting the write plan alone.

## Table layout

The raw CDC table schema is fixed and content-addressed. It is **not** a projection of the source
table's business columns: every source row is carried as canonical JSON in `payload_before_json` and
`payload_after_json`, alongside 27 flat lineage/identity columns with stable Iceberg field IDs
(`source_id`, `source_bucket`, `database_id`, `dataset_id`, `relation`, `transaction_id`,
`begin_lsn`, `commit_lsn`, `commit_timestamp_ms`, `total_order`, `operation`, `record_key`,
`idempotency_key`, `schema_fingerprint`, `schema_version`, the four `ddl_*` barrier columns,
`envelope_checksum`, the four `manifest_*` columns, `partition_key`, `epoch_id`, and `ingested_at`).
The column list plus the partition spec is hashed into a SHA-256 schema fingerprint that
provisioning and evolution check before any catalog write.

Raw tables are named `{dataset}__{schema}__{table}__raw_cdc` and partitioned by
`identity(epoch_id), identity(source_bucket)`, where `source_bucket` is a stable FNV-1a hash of
`source_id` modulo the configured bucket count. Objects land at
`{table}/epoch_id=.../source_bucket=NNNN/part-00000.parquet`.

Six unpartitioned support tables carry the completeness ledger, committed in this order:

| Commit order | Table | Content |
| ---: | --- | --- |
| 10 | `_trellara_epoch_sources` | Per-source state, LSN window, counts, checksum, lag reason. |
| 20 | `_trellara_epoch_tables` | Per-relation transaction/change counts and checksum rollup. |
| 30 | `_trellara_epoch_partitions` | Per-partition counts and checksum rollup. |
| 40 | `_trellara_quarantine` | Conflicting or blocked source evidence for the epoch. |
| 50 | `_trellara_verification` | Epoch verification status and digests. |
| 100 | `_trellara_epochs` | The single-row completeness decision consumers gate on; committed last. |

`_trellara_epochs` is written only after every raw and support receipt proves the same epoch commit,
which is what makes cross-table visibility a Trellara release-ordering contract rather than an
Iceberg guarantee.

## Features and toolchains

The default feature set exposes the catalog-independent planning and receipt contract. Enable `iceberg-rust` for catalog commits, or `production-writer` for REST catalog construction, OpenDAL S3 uploads, metadata Parquet encoding, provisioning, orchestration, and maintenance.

```toml
trellara-iceberg = { path = "../trellara-iceberg", features = ["production-writer"] }
```

The low-level runtime entry point is `commit_iceberg_table_append`; the end-to-end entry point is `write_production_iceberg_epoch`. Apache Iceberg Rust supplies optimistic transaction requirements. Trellara adds deterministic commit IDs, durable checkpoint intents/receipts, exact snapshot reconciliation, and the cross-table completeness gate.

## Development

```console
cargo test -p trellara-iceberg
cargo clippy -p trellara-iceberg --all-features --all-targets -- -D warnings
cargo check -p trellara-iceberg --features production-writer
cargo test -p trellara-iceberg --features live-catalog-tests -- --ignored
```

See [skills.md](skills.md), the [consolidated design](../../docs/DESIGN.md#lakehouse-and-iceberg-path), and the [workspace map](../README.md).
