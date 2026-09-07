use prost::Message;
use trellara_protocol::{validate_transaction_manifest, ProtocolError, TransactionManifest};
use trellara_stream::StreamMessage;

use crate::barrier::{
    validate_manifest_header_context, HeaderContext, PendingBarrierTransaction, PendingManifest,
};
use crate::worker_buffer_chunk::manifest_lists_partition;
use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn buffer_manifest(
    pending: &mut PendingBarrierTransaction,
    context: HeaderContext,
    message: StreamMessage,
) -> ApplyWorkerResult<()> {
    let manifest = TransactionManifest::decode(message.payload.as_ref())?;
    if let Some(existing) = &mut pending.manifest {
        if existing.manifest != manifest || existing.context != context {
            return Err(ApplyWorkerError::DuplicateManifest {
                transaction_id: manifest.transaction_id,
            });
        }
        validate_transaction_manifest(&manifest)?;
        validate_manifest_header_context(&context, &manifest)?;
        existing.messages.push(message);
        return Ok(());
    }

    validate_transaction_manifest(&manifest)?;
    validate_manifest_header_context(&context, &manifest)?;
    for partition_id in pending.chunks.keys() {
        if !manifest_lists_partition(&manifest, *partition_id) {
            return Err(ProtocolError::PartitionNotInManifest {
                transaction_id: manifest.transaction_id.clone(),
                partition_id: *partition_id,
            }
            .into());
        }
    }
    if let Some(pending_marker) = &pending.commit_marker {
        if !pending_marker.marker.matches_manifest(&manifest) {
            return Err(ApplyWorkerError::CommitMarkerMismatch {
                transaction_id: manifest.transaction_id.clone(),
            });
        }
    }
    pending.manifest = Some(PendingManifest {
        manifest,
        context,
        messages: vec![message],
    });
    Ok(())
}
