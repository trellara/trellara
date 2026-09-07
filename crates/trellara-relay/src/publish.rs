use tracing::debug;
use trellara_checkpoint::CheckpointStore;
use trellara_protocol::TransactionEnvelope;
use trellara_stream::StreamPublisher;

use crate::checkpoint::record_durable;
use crate::messages::messages_for_envelope;
use crate::source_ack_boundary::SourceAckBoundaryProof;
use crate::{RelayMode, RelayStep, Result};

pub(crate) async fn publish_envelope<P, C>(
    publisher: &P,
    checkpoint_store: &C,
    mode: &RelayMode,
    envelope: TransactionEnvelope,
) -> Result<RelayStep>
where
    P: StreamPublisher,
    C: CheckpointStore,
{
    let messages = messages_for_envelope(&envelope, mode)?;
    let expected_publish_messages = messages.len();
    let mut publish_acks = Vec::with_capacity(messages.len());
    for message in messages.iter().cloned() {
        publish_acks.push(publisher.publish(message).await?);
    }
    let source_ack_lsn = record_durable(checkpoint_store, &envelope).await?;
    let source_ack_boundary = SourceAckBoundaryProof::recorded(
        &envelope,
        &messages,
        &publish_acks,
        expected_publish_messages,
        &source_ack_lsn,
    )?;

    debug!(
        transaction_id = %envelope.transaction_id,
        commit_lsn = %envelope.commit_lsn,
        published_messages = publish_acks.len(),
        "published transaction envelope"
    );

    Ok(RelayStep {
        envelope,
        published_messages: messages,
        publish_acks,
        source_ack_lsn,
        source_ack_boundary,
    })
}
