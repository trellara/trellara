use std::collections::BTreeSet;

use crate::{
    partition_local_view::partition_local_view,
    partition_reconstruct::reconstruct_committed_barrier_transaction,
    partition_verify::verify_manifest_chunk, ChangeRecord, PartitionChunk, PartitionLocalView,
    ProtocolError, TransactionCommitMarker, TransactionManifest,
};

#[derive(Clone, Debug, PartialEq)]
pub enum BarrierVisibilityDecision {
    GloballyVisible {
        transaction_id: String,
        source_commit_lsn: String,
        event_count: u32,
        changes: Vec<ChangeRecord>,
    },
    Held {
        transaction_id: String,
        reason: BarrierHoldReason,
        missing_partitions: Vec<u32>,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BarrierHoldReason {
    CommitMarkerMissing,
    PartitionChunksMissing,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PartitionVisibilityDecision {
    PartitionLocalVisible {
        transaction_id: String,
        partition_id: u32,
        global_complete: bool,
        view: PartitionLocalView,
    },
}

pub fn barrier_visibility_decision(
    manifest: &TransactionManifest,
    commit_marker: Option<&TransactionCommitMarker>,
    chunks: &[PartitionChunk],
) -> Result<BarrierVisibilityDecision, ProtocolError> {
    validate_available_chunks(manifest, chunks)?;
    let missing_partitions = missing_partition_ids(manifest, chunks)?;

    let Some(commit_marker) = commit_marker else {
        return Ok(BarrierVisibilityDecision::Held {
            transaction_id: manifest.transaction_id.clone(),
            reason: BarrierHoldReason::CommitMarkerMissing,
            missing_partitions,
        });
    };

    if !missing_partitions.is_empty() {
        return Ok(BarrierVisibilityDecision::Held {
            transaction_id: manifest.transaction_id.clone(),
            reason: BarrierHoldReason::PartitionChunksMissing,
            missing_partitions,
        });
    }

    let changes = reconstruct_committed_barrier_transaction(manifest, commit_marker, chunks)?;
    Ok(BarrierVisibilityDecision::GloballyVisible {
        transaction_id: manifest.transaction_id.clone(),
        source_commit_lsn: manifest.source_commit_lsn.clone(),
        event_count: manifest.global_event_count,
        changes,
    })
}

pub fn partition_visibility_decision(
    manifest: &TransactionManifest,
    chunk: &PartitionChunk,
) -> Result<PartitionVisibilityDecision, ProtocolError> {
    let view = partition_local_view(manifest, chunk)?;
    Ok(PartitionVisibilityDecision::PartitionLocalVisible {
        transaction_id: view.transaction_id.clone(),
        partition_id: view.partition_id,
        global_complete: view.transaction_complete,
        view,
    })
}

fn validate_available_chunks(
    manifest: &TransactionManifest,
    chunks: &[PartitionChunk],
) -> Result<(), ProtocolError> {
    let mut seen = BTreeSet::new();
    for chunk in chunks {
        let expected = manifest
            .partitions
            .iter()
            .find(|partition| partition.id == chunk.partition_id)
            .ok_or_else(|| ProtocolError::PartitionNotInManifest {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: chunk.partition_id,
            })?;
        if !seen.insert(chunk.partition_id) {
            return Err(ProtocolError::DuplicatePartitionChunk {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: chunk.partition_id,
            });
        }
        verify_manifest_chunk(manifest, expected, chunk)?;
    }
    Ok(())
}

fn missing_partition_ids(
    manifest: &TransactionManifest,
    chunks: &[PartitionChunk],
) -> Result<Vec<u32>, ProtocolError> {
    let chunk_ids = chunks
        .iter()
        .map(|chunk| chunk.partition_id)
        .collect::<BTreeSet<_>>();
    let mut missing = Vec::new();
    for partition in &manifest.partitions {
        if !chunk_ids.contains(&partition.id) {
            missing.push(partition.id);
        }
    }
    Ok(missing)
}
