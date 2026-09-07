use trellara_stream::StreamMessage;

use crate::barrier::{HeaderContext, PendingBarrierTransaction};
use crate::worker_buffer_chunk::buffer_chunk;
use crate::worker_buffer_commit::buffer_commit_marker;
use crate::worker_buffer_manifest::buffer_manifest;
use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn buffer_barrier_message(
    pending: &mut PendingBarrierTransaction,
    context: HeaderContext,
    message_kind: &str,
    message: StreamMessage,
) -> ApplyWorkerResult<()> {
    match message_kind {
        "partition_chunk" | "strict_chunk" => buffer_chunk(pending, context, message),
        "manifest" => buffer_manifest(pending, context, message),
        "commit_marker" => buffer_commit_marker(pending, context, message),
        other => Err(ApplyWorkerError::UnsupportedMessageKind(other.to_string())),
    }
}
