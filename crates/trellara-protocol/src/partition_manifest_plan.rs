use crate::{
    AffectedTable, ManifestBoundaryMode, ManifestPartition, PartitionChunk, ProtocolError,
    TransactionEnvelope, TransactionManifest,
};

pub(crate) fn manifest_partition_for_chunk(
    envelope: &TransactionEnvelope,
    chunk: &PartitionChunk,
) -> Result<ManifestPartition, ProtocolError> {
    Ok(ManifestPartition {
        id: chunk.partition_id,
        event_count: manifest_count(envelope, "partition.event_count", chunk.changes.len())?,
        first_total_order: chunk
            .changes
            .iter()
            .map(|change| change.total_order)
            .min()
            .unwrap_or_default(),
        last_total_order: chunk
            .changes
            .iter()
            .map(|change| change.total_order)
            .max()
            .unwrap_or_default(),
        checksum: chunk.checksum,
    })
}

pub(crate) fn transaction_manifest(
    envelope: &TransactionEnvelope,
    partitions: Vec<ManifestPartition>,
    affected_tables: Vec<AffectedTable>,
    chunks: &[PartitionChunk],
    boundary_mode: ManifestBoundaryMode,
) -> Result<TransactionManifest, ProtocolError> {
    let global_event_count = chunks.iter().try_fold(0u32, |total, chunk| {
        let count = manifest_count(envelope, "global_event_count", chunk.changes.len())?;
        total
            .checked_add(count)
            .ok_or_else(|| ProtocolError::ManifestCountOverflow {
                transaction_id: envelope.transaction_id.clone(),
                field: "global_event_count",
                max_supported_count: u32::MAX,
            })
    })?;

    Ok(TransactionManifest {
        transaction_id: envelope.transaction_id.clone(),
        source_commit_lsn: envelope.commit_lsn.clone(),
        source_commit_timestamp_ms: envelope.commit_timestamp_ms,
        global_event_count,
        partitions,
        affected_tables,
        boundary_mode: boundary_mode as i32,
    })
}

fn manifest_count(
    envelope: &TransactionEnvelope,
    field: &'static str,
    count: usize,
) -> Result<u32, ProtocolError> {
    u32::try_from(count).map_err(|_| ProtocolError::ManifestCountOverflow {
        transaction_id: envelope.transaction_id.clone(),
        field,
        max_supported_count: u32::MAX,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StrictEnvelope, TransactionEnvelope};

    fn envelope() -> TransactionEnvelope {
        TransactionEnvelope::strict(StrictEnvelope {
            source_id: "source".to_string(),
            database_id: "db".to_string(),
            dataset_id: "dataset".to_string(),
            transaction_id: "tx-count".to_string(),
            begin_lsn: "0/16B6B00".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            commit_timestamp_ms: 1,
            changes: Vec::new(),
        })
    }

    #[test]
    fn manifest_count_accepts_u32_max() {
        assert_eq!(
            manifest_count(&envelope(), "partition.event_count", u32::MAX as usize).expect("count"),
            u32::MAX
        );
    }

    #[test]
    fn manifest_count_fails_closed_before_wraparound() {
        assert!(matches!(
            manifest_count(&envelope(), "partition.event_count", u32::MAX as usize + 1),
            Err(ProtocolError::ManifestCountOverflow {
                transaction_id,
                field: "partition.event_count",
                max_supported_count: u32::MAX,
            }) if transaction_id == "tx-count"
        ));
    }
}
