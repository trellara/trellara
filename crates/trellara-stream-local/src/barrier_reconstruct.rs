use std::path::Path;

use trellara_protocol::{
    barrier_visibility_decision, BarrierVisibilityDecision, ChangeRecord, PartitionChunk,
    TransactionCommitMarker, TransactionManifest,
};

use crate::barrier_readers::{read_commit_marker, read_manifest, read_partition_chunk};
use crate::barrier_reconstruct_request::{
    validate_barrier_reconstruction_request, LocalBarrierReconstructionRequest,
};
use crate::barrier_topics::{commit_topic, manifest_topic, partition_topic};
use crate::{LocalStreamError, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct LocalBarrierTransaction {
    pub manifest: TransactionManifest,
    pub commit_marker: TransactionCommitMarker,
    pub chunks: Vec<PartitionChunk>,
    pub changes: Vec<ChangeRecord>,
}

pub fn reconstruct_local_barrier_transaction(
    root: impl AsRef<Path>,
    request: &LocalBarrierReconstructionRequest,
) -> Result<LocalBarrierTransaction> {
    let root = root.as_ref();
    validate_barrier_reconstruction_request(request)?;
    let manifest_topic = manifest_topic(request);
    let commit_topic = commit_topic(request);
    let manifest = read_manifest(root, &manifest_topic, request.manifest_offset)?;
    let commit_marker = read_commit_marker(root, &commit_topic, request.commit_offset)?;
    let chunks = request
        .partition_offsets
        .iter()
        .map(|offset| {
            read_partition_chunk(
                root,
                &partition_topic(request, offset.partition_id),
                offset.partition_id,
                offset.offset,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let visibility = barrier_visibility_decision(&manifest, Some(&commit_marker), &chunks)?;
    let changes = match visibility {
        BarrierVisibilityDecision::GloballyVisible { changes, .. } => changes,
        BarrierVisibilityDecision::Held {
            transaction_id,
            reason,
            missing_partitions,
        } => {
            return Err(LocalStreamError::BarrierTransactionHeld {
                transaction_id,
                reason,
                missing_partitions,
            });
        }
    };

    Ok(LocalBarrierTransaction {
        manifest,
        commit_marker,
        chunks,
        changes,
    })
}
