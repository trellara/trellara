use trellara_protocol::{
    validate_commit_marker, validate_manifest_partition_ids, validate_partition_chunk,
    validate_transaction_manifest, PartitionChunk, TransactionCommitMarker, TransactionEnvelope,
    TransactionManifest,
};

use crate::{Result, StreamError};

pub(crate) fn stream_partition_id(partition_id: u32) -> Result<i32> {
    i32::try_from(partition_id).map_err(|_| StreamError::InvalidStreamPartition {
        partition_id,
        max_supported: i32::MAX as u32,
    })
}

pub(crate) fn validate_envelope_metadata(envelope: &TransactionEnvelope) -> Result<()> {
    envelope.validate()?;
    envelope.verify_checksum()?;
    Ok(())
}

pub(crate) fn validate_manifest_payload(
    envelope: &TransactionEnvelope,
    manifest: &TransactionManifest,
) -> Result<()> {
    validate_manifest_partition_ids(manifest)?;
    validate_field(
        "transaction_id",
        &envelope.transaction_id,
        &manifest.transaction_id,
    )?;
    validate_field(
        "commit_lsn",
        &envelope.commit_lsn,
        &manifest.source_commit_lsn,
    )?;
    validate_field(
        "commit_timestamp_ms",
        &envelope.commit_timestamp_ms.to_string(),
        &manifest.source_commit_timestamp_ms.to_string(),
    )?;
    validate_field(
        "global_event_count",
        &envelope.changes.len().to_string(),
        &manifest.global_event_count.to_string(),
    )?;
    validate_transaction_manifest(manifest)?;
    Ok(())
}

pub(crate) fn validate_chunk_payload(
    envelope: &TransactionEnvelope,
    chunk: &PartitionChunk,
) -> Result<()> {
    validate_partition_chunk(chunk)?;
    validate_field(
        "transaction_id",
        &envelope.transaction_id,
        &chunk.transaction_id,
    )?;
    for change in &chunk.changes {
        validate_field(
            "change.transaction_id",
            &envelope.transaction_id,
            &change.transaction_id,
        )?;
        validate_chunk_change_matches_envelope(envelope, change)?;
    }
    Ok(())
}

pub(crate) fn validate_marker_payload(
    envelope: &TransactionEnvelope,
    marker: &TransactionCommitMarker,
) -> Result<()> {
    validate_commit_marker(marker)?;
    validate_field(
        "transaction_id",
        &envelope.transaction_id,
        &marker.transaction_id,
    )?;
    validate_field(
        "commit_lsn",
        &envelope.commit_lsn,
        &marker.source_commit_lsn,
    )?;
    validate_field(
        "commit_timestamp_ms",
        &envelope.commit_timestamp_ms.to_string(),
        &marker.source_commit_timestamp_ms.to_string(),
    )?;
    validate_field(
        "global_event_count",
        &envelope.changes.len().to_string(),
        &marker.global_event_count.to_string(),
    )?;
    if marker.participating_partition_count == 0 {
        return Err(StreamError::BarrierPayloadMismatch {
            field: "participating_partition_count",
            envelope: "nonzero".to_string(),
            payload: marker.participating_partition_count.to_string(),
        });
    }
    Ok(())
}

fn validate_field(field: &'static str, envelope: &str, payload: &str) -> Result<()> {
    if envelope == payload {
        Ok(())
    } else {
        Err(StreamError::BarrierPayloadMismatch {
            field,
            envelope: envelope.to_string(),
            payload: payload.to_string(),
        })
    }
}

fn validate_chunk_change_matches_envelope(
    envelope: &TransactionEnvelope,
    change: &trellara_protocol::ChangeRecord,
) -> Result<()> {
    let Some(expected) = envelope
        .changes
        .iter()
        .find(|expected| expected.total_order == change.total_order)
    else {
        return Err(StreamError::BarrierPayloadMismatch {
            field: "chunk.change.total_order",
            envelope: envelope
                .changes
                .iter()
                .map(|change| change.total_order.to_string())
                .collect::<Vec<_>>()
                .join(","),
            payload: change.total_order.to_string(),
        });
    };
    if expected == change {
        return Ok(());
    }
    Err(StreamError::BarrierPayloadMismatch {
        field: "chunk.change",
        envelope: expected.total_order.to_string(),
        payload: change.total_order.to_string(),
    })
}
