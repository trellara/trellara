# trellara-protocol

`trellara-protocol` is Trellara's transport- and storage-independent domain contract. Every source, relay, stream adapter, target, and verifier agrees on the types and invariants defined here.

## Responsibilities

- Define the protocol version and transaction envelope.
- Model rows, changes, relation identities, schema versions, checkpoints, and DDL events.
- Validate envelopes, DDL ordering, manifests, chunks, and commit markers.
- Define strict, strict-chunked, and partitioned transaction boundaries.
- Plan and reconstruct partition-local views without weakening atomic visibility.
- Produce deterministic routing, checksums, LSN formatting, and idempotency keys.

## Boundary vocabulary

Three runtime shapes, but two different enums, and they are easy to confuse:

| Layer | Type | Values |
| --- | --- | --- |
| Flow configuration (`dataset.mode`, owned by `trellara-cli`) | `DatasetMode` | `strict_transaction_order`, `partitioned_scale_mode` |
| Wire manifests (this crate) | `ManifestBoundaryMode` | `Unspecified = 0`, `StrictChunkedTransactionOrder = 1`, `PartitionedScale = 2` |

There is no `strict_chunked` dataset mode. Strict chunking is a bounded form of
`strict_transaction_order`, switched on for the whole flow by
`dataset.strict_chunking.max_changes_per_chunk`. The choice is per flow, not per transaction: with
chunking configured, `RelayMode::StrictChunked` splits every transaction's changes into
`max_changes_per_chunk`-sized ordered chunks and always emits a manifest plus a manifest-bound
commit marker after them, even for a one-change transaction. Without it, `RelayMode::Strict`
publishes exactly one envelope message and no manifest.

`ManifestBoundaryMode::status_mode()` is the string form used in operator output:
`strict_chunked_transaction_order`, `partitioned_scale_mode`, and `manifest_barrier_transaction`
for `Unspecified`. `TrellaraConfig::status_mode()` reports the same three strings from
configuration.

`PROTOCOL_VERSION` is `1`. The canonical ordered event key is
`idempotency_key(source_id, commit_lsn, transaction_id, total_order)`, formatted
`{source_id}:{commit_lsn}:{transaction_id}:{total_order}`. `TransactionBoundaryKey` additionally
carries an optional `database_id`, which participates in `Display` but not in the event key.

## Boundaries

This crate does not connect to PostgreSQL, publish to a broker, write checkpoints, apply SQL, or perform operator I/O. It must remain deterministic and usable without a runtime. Transport headers belong in `trellara-stream`; PostgreSQL decoding belongs in `trellara-pg-capture`.

Changes to serialized types or validation rules are compatibility changes. Update protocol fixtures, round-trip tests, malformed-input tests, generated protobuf compatibility evidence, and the consolidated design when behavior changes.

## Key entry points

- `TransactionEnvelope` and transaction-boundary types
- `PartitionChunk`, transaction manifests, and commit markers
- DDL propagation envelopes, ordered schema operations, and post-DDL DML replay projections
- Partition routing and rebalance plans
- `validate_transaction_manifest` and partition reconstruction helpers
- `idempotency_key`, `parse_lsn`, and `format_lsn`

## Development

```console
cargo test -p trellara-protocol
cargo clippy -p trellara-protocol --all-targets -- -D warnings
```

See [skills.md](skills.md) for package-specific development standards and the [workspace map](../README.md) for surrounding packages.
