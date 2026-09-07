use trellara_stream::StreamMessage;

use crate::barrier::PendingBarrierTransaction;
use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn ordered_ack_messages(
    pending_transaction: &PendingBarrierTransaction,
    transaction_key: &str,
) -> ApplyWorkerResult<Vec<StreamMessage>> {
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
    let mut messages = Vec::new();

    for partition in &manifest.partitions {
        let pending_chunk = pending_transaction
            .chunks
            .get(&partition.id)
            .ok_or_else(|| ApplyWorkerError::ReadyTransactionMissingChunk {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: partition.id,
            })?;
        messages.extend(pending_chunk.messages.clone());
    }
    messages.extend(pending_manifest.messages.clone());
    messages.extend(pending_commit_marker.messages.clone());

    Ok(messages)
}
