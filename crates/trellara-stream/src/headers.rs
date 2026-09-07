use trellara_protocol::{
    PartitionChunk, TransactionCommitMarker, TransactionEnvelope, PROTOCOL_VERSION,
};

use crate::header_ddl::ddl_headers;
use crate::header_metadata::schema_versions_header;
use crate::StreamHeader;

pub fn envelope_headers(envelope: &TransactionEnvelope) -> Vec<StreamHeader> {
    let partitioned_scale_readiness = envelope.partitioned_scale_readiness();
    let mut headers = vec![
        StreamHeader::new("trellara.protocol_version", PROTOCOL_VERSION.to_string()),
        StreamHeader::new("trellara.source_id", envelope.source_id.clone()),
        StreamHeader::new("trellara.dataset_id", envelope.dataset_id.clone()),
        StreamHeader::new("trellara.database_id", envelope.database_id.clone()),
        StreamHeader::new("trellara.transaction_id", envelope.transaction_id.clone()),
        StreamHeader::new("trellara.commit_lsn", envelope.commit_lsn.clone()),
        StreamHeader::new(
            "trellara.commit_timestamp_ms",
            envelope.commit_timestamp_ms.to_string(),
        ),
        StreamHeader::new(
            "trellara.schema_version_count",
            envelope.schema_versions.len().to_string(),
        ),
        StreamHeader::new(
            "trellara.ddl_event_count",
            envelope.ddl_events.len().to_string(),
        ),
        StreamHeader::new(
            "trellara.partitioned_scale_decision",
            partitioned_scale_readiness.decision.to_string(),
        ),
        StreamHeader::new(
            "trellara.partition_parallel_safe",
            partitioned_scale_readiness
                .partition_parallel_safe
                .to_string(),
        ),
        StreamHeader::new(
            "trellara.requires_ddl_barrier",
            partitioned_scale_readiness.requires_ddl_barrier.to_string(),
        ),
        StreamHeader::new(
            "trellara.dml_replay_after_ddl_barrier_required",
            partitioned_scale_readiness
                .dml_replay_after_ddl_barrier_required
                .to_string(),
        ),
        StreamHeader::new(
            "trellara.partitioned_scale_reason",
            partitioned_scale_readiness.reason,
        ),
    ];

    if !envelope.schema_versions.is_empty() {
        headers.push(StreamHeader::new(
            "trellara.schema_versions",
            schema_versions_header(envelope),
        ));
    }
    headers.extend(ddl_headers(envelope));

    headers
}

pub fn commit_marker_headers(
    envelope: &TransactionEnvelope,
    marker: &TransactionCommitMarker,
) -> Vec<StreamHeader> {
    let mut headers = envelope_headers(envelope);
    headers.push(StreamHeader::new("trellara.message_kind", "commit_marker"));
    headers.push(StreamHeader::new(
        "trellara.global_event_count",
        marker.global_event_count.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partition_count",
        marker.participating_partition_count.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.manifest_checksum",
        marker.manifest_checksum.to_string(),
    ));
    headers
}

pub fn partition_chunk_headers(
    envelope: &TransactionEnvelope,
    chunk: &PartitionChunk,
) -> Vec<StreamHeader> {
    chunk_headers(envelope, chunk, "partition_chunk")
}

pub fn strict_chunk_headers(
    envelope: &TransactionEnvelope,
    chunk: &PartitionChunk,
) -> Vec<StreamHeader> {
    chunk_headers(envelope, chunk, "strict_chunk")
}

fn chunk_headers(
    envelope: &TransactionEnvelope,
    chunk: &PartitionChunk,
    message_kind: &'static str,
) -> Vec<StreamHeader> {
    let mut headers = envelope_headers(envelope);
    headers.push(StreamHeader::new("trellara.message_kind", message_kind));
    headers.push(StreamHeader::new(
        "trellara.partition_id",
        chunk.partition_id.to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partition_event_count",
        chunk.changes.len().to_string(),
    ));
    headers.push(StreamHeader::new(
        "trellara.partition_checksum",
        chunk.checksum.to_string(),
    ));
    headers
}
