# trellara-stream-local

`trellara-stream-local` is Trellara's brokerless durable stream adapter. It stores stream messages in append-only topic segments with recoverable indexes and durable consumer cursors.

## Responsibilities

- Append framed messages with explicit durability modes and publish acknowledgements.
- Validate frame lengths, checksums, headers, and topic paths.
- Maintain and rebuild topic indexes after interruption.
- Persist monotonic consumer cursors and support explicit administrative seek policies.
- Detect and recover from torn tails without accepting corrupt committed frames.
- Inspect local topics, offsets, cursors, and message metadata
- Produce fsync-backed publish-ACK and source-ACK durability proofs for downstream relay consumption
- Report recovery readiness (blockers and topic evidence) after an interrupted process before resuming.
- Reconstruct strict/partitioned barrier transactions from local messages.

## On-disk format

Segments are append-only frames: a four-byte magic, a `u32` body length, the encoded body, and a
`u32` CRC32 of the body. The current magic is `TLG2`; `TLG1` frames are still readable so an
existing log survives an upgrade, but every new frame is written as `TLG2`. Any other magic, a
length past the configured maximum, or a checksum mismatch is a `CorruptFrame` error rather than a
skipped record. A short read at a frame boundary is a torn tail and truncates to the last complete
valid frame.

A sidecar index maps logical offsets to file positions and is rebuilt from validated frames when it
is missing or stale. Consumer cursors live outside the log, one per named consumer, and are
persisted atomically before an acknowledgement returns.

`LocalDurability::Fsync` is the default for both publish and cursor acknowledgement, so a
`PublishAck` means the bytes reached stable storage — not that they reached the page cache.

## Boundaries

This crate implements the `trellara-stream` port for local storage. It does not define transaction semantics, source-ACK sequencing, target apply, or Kafka behavior.

Filesystem paths and bytes are untrusted input. Do not allow topic names to escape the configured root, and do not report a durable publish acknowledgement before the required flush/sync boundary succeeds.

## Key entry points

- `LocalPublisher` / `LocalPublisherConfig` and `LocalConsumer` / `LocalConsumerConfig`, which
  implement `StreamPublisher` and `StreamConsumer`
- `LocalDurability` (`Fsync` by default, `Buffered`) and `LocalAckOutcome`
- `read_local_message_at`
- `set_local_cursor` and `set_local_cursor_with_policy`
- `inspect_local_stream` and `inspect_local_recovery`
- `reconstruct_local_barrier_transaction` and `LocalBarrierReconstructionRequest`
- `local_publish_ack_proof` and `local_source_ack_durability_proof`, with their
  `LOCAL_PUBLISH_ACK_PROOF_CONTRACT` and `LOCAL_SOURCE_ACK_DURABILITY_CONTRACT` labels

## Development

```console
cargo test -p trellara-stream-local
cargo clippy -p trellara-stream-local --all-targets -- -D warnings
```

See [skills.md](skills.md) and the [workspace map](../README.md).
