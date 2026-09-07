use trellara_protocol::{
    PartitionChunk, PartitionPlan, TransactionCommitMarker, TransactionEnvelope,
};
use trellara_stream::StreamMessage;

use crate::Result;

pub(crate) fn barrier_messages_for_plan(
    envelope: &TransactionEnvelope,
    plan: &PartitionPlan,
    chunk_message: fn(
        &TransactionEnvelope,
        &PartitionChunk,
    ) -> trellara_stream::Result<StreamMessage>,
) -> Result<Vec<StreamMessage>> {
    let mut messages = plan
        .chunks
        .iter()
        .map(|chunk| chunk_message(envelope, chunk))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    messages.push(StreamMessage::transaction_manifest(
        envelope,
        &plan.manifest,
    )?);
    messages.push(StreamMessage::commit_marker(
        envelope,
        &TransactionCommitMarker::from_manifest(&plan.manifest)?,
    )?);
    Ok(messages)
}
