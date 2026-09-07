use std::collections::{BTreeMap, BTreeSet};

use crate::{
    partition_operation_order::operation_order, partition_verify::verify_manifest_chunk,
    validate_commit_marker, validate_transaction_manifest, ChangeRecord, PartitionChunk,
    ProtocolError, TransactionCommitMarker, TransactionManifest,
};

pub fn reconstruct_barrier_transaction(
    manifest: &TransactionManifest,
    chunks: &[PartitionChunk],
) -> Result<Vec<ChangeRecord>, ProtocolError> {
    validate_transaction_manifest(manifest)?;
    let expected_partitions = manifest
        .partitions
        .iter()
        .map(|partition| (partition.id, partition))
        .collect::<BTreeMap<_, _>>();
    let mut chunks_by_partition = BTreeMap::new();
    for chunk in chunks {
        if !expected_partitions.contains_key(&chunk.partition_id) {
            return Err(ProtocolError::PartitionNotInManifest {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: chunk.partition_id,
            });
        }

        if chunks_by_partition
            .insert(chunk.partition_id, chunk)
            .is_some()
        {
            return Err(ProtocolError::DuplicatePartitionChunk {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: chunk.partition_id,
            });
        }
    }
    let mut changes = Vec::new();

    for expected in &manifest.partitions {
        let chunk = chunks_by_partition.get(&expected.id).ok_or_else(|| {
            ProtocolError::MissingPartitionChunk {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: expected.id,
            }
        })?;

        verify_manifest_chunk(manifest, expected, chunk)?;

        changes.extend(chunk.changes.clone());
    }
    let mut ordered_changes = ordered_reconstructed_changes(changes)?;
    ordered_changes
        .sort_by_key(|(total_order, operation_order, _)| (*total_order, *operation_order));

    Ok(ordered_changes
        .into_iter()
        .map(|(_, _, change)| change)
        .collect())
}

pub fn reconstruct_committed_barrier_transaction(
    manifest: &TransactionManifest,
    commit_marker: &TransactionCommitMarker,
    chunks: &[PartitionChunk],
) -> Result<Vec<ChangeRecord>, ProtocolError> {
    validate_commit_marker(commit_marker)?;
    if !commit_marker.matches_manifest(manifest) {
        return Err(ProtocolError::CommitMarkerManifestMismatch {
            transaction_id: manifest.transaction_id.clone(),
        });
    }

    reconstruct_barrier_transaction(manifest, chunks)
}

pub fn validate_manifest_partition_ids(
    manifest: &TransactionManifest,
) -> Result<(), ProtocolError> {
    let mut seen = BTreeSet::new();
    for partition in &manifest.partitions {
        if !seen.insert(partition.id) {
            return Err(ProtocolError::DuplicateManifestPartition {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: partition.id,
            });
        }
    }
    Ok(())
}

fn ordered_reconstructed_changes(
    changes: Vec<ChangeRecord>,
) -> Result<Vec<(u32, u8, ChangeRecord)>, ProtocolError> {
    let mut seen_ordered_fragments = BTreeSet::new();
    let mut ordered = Vec::with_capacity(changes.len());
    for change in changes {
        let operation_order = operation_order(change.total_order, change.operation)?;
        if !seen_ordered_fragments.insert((change.total_order, operation_order)) {
            return Err(ProtocolError::DuplicateTransactionEventOrder {
                total_order: change.total_order,
            });
        }
        ordered.push((change.total_order, operation_order, change));
    }
    Ok(ordered)
}
