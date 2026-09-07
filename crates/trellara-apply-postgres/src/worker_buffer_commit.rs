use prost::Message;
use trellara_protocol::{validate_commit_marker, TransactionCommitMarker};
use trellara_stream::StreamMessage;

use crate::barrier::{
    validate_marker_header_context, HeaderContext, PendingBarrierTransaction, PendingCommitMarker,
};
use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn buffer_commit_marker(
    pending: &mut PendingBarrierTransaction,
    context: HeaderContext,
    message: StreamMessage,
) -> ApplyWorkerResult<()> {
    let marker = TransactionCommitMarker::decode(message.payload.as_ref())?;
    validate_commit_marker(&marker)?;
    validate_marker_header_context(&context, &marker)?;
    if let Some(manifest) = &pending.manifest {
        if !marker.matches_manifest(&manifest.manifest) {
            return Err(ApplyWorkerError::CommitMarkerMismatch {
                transaction_id: manifest.manifest.transaction_id.clone(),
            });
        }
    }
    if let Some(existing) = &mut pending.commit_marker {
        if existing.marker != marker {
            return Err(ApplyWorkerError::CommitMarkerMismatch {
                transaction_id: context.transaction_id,
            });
        }
        existing.messages.push(message);
    } else {
        pending.commit_marker = Some(PendingCommitMarker {
            marker,
            messages: vec![message],
        });
    }
    Ok(())
}
