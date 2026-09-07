use crate::{
    partition_operation_order::operation_order,
    partition_reconstruct::validate_manifest_partition_ids,
    partition_verify::verify_manifest_chunk, validate_transaction_manifest, ChangeRecord,
    PartitionChunk, PartitionLocalView, ProtocolError, TransactionManifest,
};

pub fn partition_local_view(
    manifest: &TransactionManifest,
    chunk: &PartitionChunk,
) -> Result<PartitionLocalView, ProtocolError> {
    validate_manifest_partition_ids(manifest)?;
    if chunk.transaction_id != manifest.transaction_id {
        return Err(ProtocolError::PartitionTransactionMismatch {
            partition_id: chunk.partition_id,
            expected: manifest.transaction_id.clone(),
            actual: chunk.transaction_id.clone(),
        });
    }

    let expected = manifest
        .partitions
        .iter()
        .find(|partition| partition.id == chunk.partition_id)
        .ok_or_else(|| ProtocolError::PartitionNotInManifest {
            transaction_id: manifest.transaction_id.clone(),
            partition_id: chunk.partition_id,
        })?;

    validate_transaction_manifest(manifest)?;
    verify_manifest_chunk(manifest, expected, chunk)?;
    validate_change_operations(&chunk.changes)?;

    Ok(PartitionLocalView {
        transaction_id: manifest.transaction_id.clone(),
        source_commit_lsn: manifest.source_commit_lsn.clone(),
        source_commit_timestamp_ms: manifest.source_commit_timestamp_ms,
        partition_id: chunk.partition_id,
        partition_event_count: expected.event_count,
        global_event_count: manifest.global_event_count,
        participating_partition_count: participating_partition_count(manifest)?,
        first_total_order: expected.first_total_order,
        last_total_order: expected.last_total_order,
        transaction_complete: expected.event_count == manifest.global_event_count
            && manifest.partitions.len() == 1,
        changes: chunk.changes.clone(),
    })
}

fn participating_partition_count(manifest: &TransactionManifest) -> Result<u32, ProtocolError> {
    u32::try_from(manifest.partitions.len()).map_err(|_| ProtocolError::ManifestCountOverflow {
        transaction_id: manifest.transaction_id.clone(),
        field: "participating_partition_count",
        max_supported_count: u32::MAX,
    })
}

fn validate_change_operations(changes: &[ChangeRecord]) -> Result<(), ProtocolError> {
    for change in changes {
        operation_order(change.total_order, change.operation)?;
    }
    Ok(())
}
