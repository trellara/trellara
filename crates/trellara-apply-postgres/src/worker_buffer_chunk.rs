use prost::Message;
use trellara_protocol::{
    validate_partition_chunk, PartitionChunk, ProtocolError, TransactionManifest,
};
use trellara_stream::StreamMessage;

use crate::barrier::{
    same_partition_chunk, validate_chunk_header_context, HeaderContext, PendingBarrierTransaction,
    PendingChunk,
};
use crate::ApplyWorkerResult;

pub(crate) fn buffer_chunk(
    pending: &mut PendingBarrierTransaction,
    context: HeaderContext,
    message: StreamMessage,
) -> ApplyWorkerResult<()> {
    let chunk = PartitionChunk::decode(message.payload.as_ref())?;
    validate_partition_chunk(&chunk)?;
    validate_chunk_header_context(&context, &chunk)?;
    if let Some(manifest) = &pending.manifest {
        if !manifest_lists_partition(&manifest.manifest, chunk.partition_id) {
            return Err(ProtocolError::PartitionNotInManifest {
                transaction_id: manifest.manifest.transaction_id.clone(),
                partition_id: chunk.partition_id,
            }
            .into());
        }
    }
    if let Some(existing) = pending.chunks.get_mut(&chunk.partition_id) {
        if !same_partition_chunk(&existing.chunk, &chunk) {
            return Err(ProtocolError::DuplicatePartitionChunk {
                transaction_id: context.transaction_id,
                partition_id: chunk.partition_id,
            }
            .into());
        }
        existing.messages.push(message);
    } else {
        pending.chunks.insert(
            chunk.partition_id,
            PendingChunk {
                chunk,
                messages: vec![message],
            },
        );
    }
    Ok(())
}

pub(crate) fn manifest_lists_partition(manifest: &TransactionManifest, partition_id: u32) -> bool {
    manifest
        .partitions
        .iter()
        .any(|partition| partition.id == partition_id)
}
