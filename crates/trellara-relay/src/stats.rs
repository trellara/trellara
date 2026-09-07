use trellara_protocol::TransactionEnvelope;
use trellara_stream::{PublishAck, StreamMessage};

use crate::source_ack_proof_validation::validate_step_source_ack_proof;
use crate::{checked_relay_stat_add, RelayError, Result, SourceAckBoundaryProof};

#[derive(Clone, Debug, PartialEq)]
pub struct RelayStep {
    pub envelope: TransactionEnvelope,
    pub published_messages: Vec<StreamMessage>,
    pub publish_acks: Vec<PublishAck>,
    pub source_ack_lsn: String,
    pub source_ack_boundary: SourceAckBoundaryProof,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RelayRunStats {
    pub published_transactions: u64,
    pub published_messages: u64,
    pub last_commit_lsn: Option<String>,
    pub last_ack: Option<PublishAck>,
    pub last_published_messages: Vec<StreamMessage>,
    pub last_publish_acks: Vec<PublishAck>,
    pub last_source_ack_boundary: Option<SourceAckBoundaryProof>,
}

impl RelayRunStats {
    pub(crate) fn record_step(&mut self, step: RelayStep) -> Result<()> {
        validate_step_source_ack_proof(&step)?;
        self.published_transactions =
            checked_relay_stat_add(self.published_transactions, 1, "published_transactions")?;
        let published_messages =
            u64::try_from(step.publish_acks.len()).map_err(|_| RelayError::StatOverflow {
                field: "published_messages",
            })?;
        self.published_messages = checked_relay_stat_add(
            self.published_messages,
            published_messages,
            "published_messages",
        )?;
        self.last_commit_lsn = Some(step.envelope.commit_lsn);
        self.last_ack = step.publish_acks.last().cloned();
        self.last_published_messages = step.published_messages;
        self.last_publish_acks = step.publish_acks;
        self.last_source_ack_boundary = Some(step.source_ack_boundary);
        Ok(())
    }
}
