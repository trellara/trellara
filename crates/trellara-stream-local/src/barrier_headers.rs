use trellara_protocol::{PartitionChunk, TransactionCommitMarker, TransactionManifest};
use trellara_stream::StreamMessage;

use crate::{barrier_header_payload::validate_header_payload, Result};

pub(crate) fn validate_manifest_headers(
    message: &StreamMessage,
    manifest: &TransactionManifest,
    offset: i64,
) -> Result<()> {
    validate_partition_parallel_header(message, offset)?;
    validate_header_payload(
        message,
        offset,
        "transaction_id",
        manifest.transaction_id.clone(),
    )?;
    validate_header_payload(
        message,
        offset,
        "commit_lsn",
        manifest.source_commit_lsn.clone(),
    )?;
    validate_header_payload(
        message,
        offset,
        "global_event_count",
        manifest.global_event_count.to_string(),
    )?;
    validate_header_payload(
        message,
        offset,
        "partition_count",
        manifest.partitions.len().to_string(),
    )
}

pub(crate) fn validate_commit_marker_headers(
    message: &StreamMessage,
    marker: &TransactionCommitMarker,
    offset: i64,
) -> Result<()> {
    validate_partition_parallel_header(message, offset)?;
    validate_header_payload(
        message,
        offset,
        "transaction_id",
        marker.transaction_id.clone(),
    )?;
    validate_header_payload(
        message,
        offset,
        "commit_lsn",
        marker.source_commit_lsn.clone(),
    )?;
    validate_header_payload(
        message,
        offset,
        "global_event_count",
        marker.global_event_count.to_string(),
    )?;
    validate_header_payload(
        message,
        offset,
        "partition_count",
        marker.participating_partition_count.to_string(),
    )?;
    validate_header_payload(
        message,
        offset,
        "manifest_checksum",
        marker.manifest_checksum.to_string(),
    )
}

pub(crate) fn validate_chunk_headers(
    message: &StreamMessage,
    chunk: &PartitionChunk,
    offset: i64,
) -> Result<()> {
    validate_partition_parallel_header(message, offset)?;
    validate_header_payload(
        message,
        offset,
        "partition_id",
        chunk.partition_id.to_string(),
    )?;
    validate_header_payload(
        message,
        offset,
        "partition_event_count",
        chunk.changes.len().to_string(),
    )?;
    validate_header_payload(
        message,
        offset,
        "partition_checksum",
        chunk.checksum.to_string(),
    )
}

fn validate_partition_parallel_header(message: &StreamMessage, offset: i64) -> Result<()> {
    validate_header_payload(
        message,
        offset,
        "partitioned_scale_decision",
        "partition_parallel_dml",
    )
}
