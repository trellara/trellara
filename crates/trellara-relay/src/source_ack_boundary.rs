use trellara_protocol::{format_lsn, parse_lsn, TransactionEnvelope};
use trellara_stream::{PublishAck, StreamMessage};

use crate::publish_proof::RelayPublishProof;
use crate::{RelayError, Result};

pub const SOURCE_ACK_BOUNDARY_CONTRACT: &str =
    "source acknowledgement advances only after every Trellara publish ack is durable and the source checkpoint records a durable LSN";

pub use crate::publish_proof::PublishDestination;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAckBoundaryProof {
    pub contract: &'static str,
    pub source_id: String,
    pub dataset_id: String,
    pub commit_lsn: String,
    pub source_ack_lsn: String,
    pub expected_publish_messages: usize,
    pub durable_publish_acks: usize,
    pub expected_publish_destination_count: usize,
    pub durable_publish_destination_count: usize,
    pub expected_publish_destinations: Vec<PublishDestination>,
    pub durable_publish_destinations: Vec<PublishDestination>,
    pub publish_destinations_match: bool,
    pub all_publish_acks_durable: bool,
    pub last_publish_ack: Option<PublishAck>,
    pub durable_lsn_covers_commit: bool,
    pub checkpoint_recorded_before_source_ack: bool,
}

impl SourceAckBoundaryProof {
    pub(crate) fn recorded(
        envelope: &TransactionEnvelope,
        messages: &[StreamMessage],
        publish_acks: &[PublishAck],
        expected_publish_messages: usize,
        source_ack_lsn: &str,
    ) -> Result<Self> {
        if publish_acks.len() != expected_publish_messages {
            return Err(RelayError::SourceAckBeforeDurablePublish {
                expected_publish_messages,
                durable_publish_acks: publish_acks.len(),
            });
        }
        let publish_proof =
            RelayPublishProof::recorded(messages, publish_acks, expected_publish_messages)?;
        let expected_publish_destination_count = publish_proof.expected_publish_destinations.len();
        let durable_publish_destination_count = publish_proof.durable_publish_destinations.len();
        let publish_destinations_match = publish_proof.expected_publish_destinations
            == publish_proof.durable_publish_destinations;
        let boundary_key = envelope.boundary_key()?;
        let commit_lsn = boundary_key.commit_lsn;
        let source_ack_lsn = canonical_lsn(source_ack_lsn)?;
        if parse_lsn(&source_ack_lsn)? < parse_lsn(&commit_lsn)? {
            return Err(RelayError::SourceAckLsnBehindCommit {
                commit_lsn,
                source_ack_lsn,
            });
        }

        Ok(Self {
            contract: SOURCE_ACK_BOUNDARY_CONTRACT,
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            commit_lsn,
            source_ack_lsn,
            expected_publish_messages: publish_proof.expected_publish_messages,
            durable_publish_acks: publish_proof.durable_publish_acks,
            expected_publish_destination_count,
            durable_publish_destination_count,
            expected_publish_destinations: publish_proof.expected_publish_destinations,
            durable_publish_destinations: publish_proof.durable_publish_destinations,
            publish_destinations_match,
            all_publish_acks_durable: publish_proof.all_publish_acks_durable,
            last_publish_ack: publish_proof.last_publish_ack,
            durable_lsn_covers_commit: true,
            checkpoint_recorded_before_source_ack: true,
        })
    }
}

fn canonical_lsn(lsn: &str) -> Result<String> {
    Ok(format_lsn(parse_lsn(lsn)?))
}

#[cfg(test)]
#[path = "tests/tests_source_ack_boundary.rs"]
mod tests;
