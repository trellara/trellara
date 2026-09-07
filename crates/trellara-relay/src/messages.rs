use trellara_protocol::{
    plan_partitioned_transaction, plan_strict_chunked_transaction, TransactionEnvelope,
};
use trellara_stream::StreamMessage;

use crate::messages_barrier::barrier_messages_for_plan;
use crate::{RelayMode, Result};

pub(crate) fn messages_for_envelope(
    envelope: &TransactionEnvelope,
    mode: &RelayMode,
) -> Result<Vec<StreamMessage>> {
    match mode {
        RelayMode::Strict => Ok(vec![StreamMessage::strict_transaction(envelope)?]),
        RelayMode::StrictChunked(config) => {
            let plan = plan_strict_chunked_transaction(envelope, config)?;
            barrier_messages_for_plan(envelope, &plan, StreamMessage::strict_chunk)
        }
        RelayMode::Partitioned(config) => {
            let plan = plan_partitioned_transaction(envelope, config)?;
            barrier_messages_for_plan(envelope, &plan, StreamMessage::partition_chunk)
        }
    }
}
