use trellara_protocol::{
    barrier_visibility_decision, BarrierVisibilityDecision, StrictEnvelope, TransactionEnvelope,
};

use crate::barrier::PendingBarrierTransaction;
use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn build_ready_envelope(
    pending_transaction: &PendingBarrierTransaction,
    transaction_key: &str,
) -> ApplyWorkerResult<TransactionEnvelope> {
    let pending_manifest = pending_transaction.manifest.as_ref().ok_or_else(|| {
        ApplyWorkerError::ReadyTransactionMissingManifest {
            transaction_key: transaction_key.to_string(),
        }
    })?;
    let pending_commit_marker = pending_transaction.commit_marker.as_ref().ok_or_else(|| {
        ApplyWorkerError::ReadyTransactionMissingCommitMarker {
            transaction_key: transaction_key.to_string(),
        }
    })?;
    let manifest = &pending_manifest.manifest;
    let context = &pending_manifest.context;
    for partition_id in pending_transaction.chunks.keys() {
        if !manifest
            .partitions
            .iter()
            .any(|partition| partition.id == *partition_id)
        {
            return Err(trellara_protocol::ProtocolError::PartitionNotInManifest {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: *partition_id,
            }
            .into());
        }
    }
    let chunks = manifest
        .partitions
        .iter()
        .map(|partition| {
            pending_transaction
                .chunks
                .get(&partition.id)
                .map(|pending_chunk| pending_chunk.chunk.clone())
                .ok_or_else(|| ApplyWorkerError::ReadyTransactionMissingChunk {
                    transaction_id: manifest.transaction_id.clone(),
                    partition_id: partition.id,
                })
        })
        .collect::<ApplyWorkerResult<Vec<_>>>()?;
    let visibility =
        barrier_visibility_decision(manifest, Some(&pending_commit_marker.marker), &chunks)?;
    let changes = match visibility {
        BarrierVisibilityDecision::GloballyVisible { changes, .. } => changes,
        BarrierVisibilityDecision::Held {
            transaction_id,
            reason,
            missing_partitions,
        } => {
            return Err(ApplyWorkerError::BarrierTransactionHeld {
                transaction_id,
                reason,
                missing_partitions,
            });
        }
    };
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: context.source_id.clone(),
        database_id: context.database_id.clone(),
        dataset_id: context.dataset_id.clone(),
        transaction_id: manifest.transaction_id.clone(),
        begin_lsn: String::new(),
        commit_lsn: manifest.source_commit_lsn.clone(),
        commit_timestamp_ms: manifest.source_commit_timestamp_ms,
        changes,
    });
    envelope.manifest = Some(manifest.clone());
    envelope.finalize_checksum();
    envelope.validate()?;

    Ok(envelope)
}
