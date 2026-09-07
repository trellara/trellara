use std::collections::BTreeMap;

use trellara_stream::{PublishAck, StreamMessage};

use crate::{RelayError, Result};

pub const RELAY_PUBLISH_PROOF_CONTRACT: &str =
    "relay_records_ordered_durable_publish_acks_before_source_ack";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelayPublishProof {
    pub contract: &'static str,
    pub expected_publish_messages: usize,
    pub durable_publish_acks: usize,
    pub expected_publish_destinations: Vec<PublishDestination>,
    pub durable_publish_destinations: Vec<PublishDestination>,
    pub last_publish_ack: Option<PublishAck>,
    pub ordered_ack_offsets: Vec<i64>,
    pub all_publish_acks_durable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishDestination {
    pub topic: String,
    pub partition: i32,
}

impl RelayPublishProof {
    pub(crate) fn recorded(
        messages: &[StreamMessage],
        publish_acks: &[PublishAck],
        expected_publish_messages: usize,
    ) -> Result<Self> {
        if publish_acks.len() != expected_publish_messages {
            return Err(RelayError::SourceAckBeforeDurablePublish {
                expected_publish_messages,
                durable_publish_acks: publish_acks.len(),
            });
        }

        let expected_publish_destinations = expected_destinations(messages)?;
        let durable_publish_destinations = durable_destinations(publish_acks);
        if expected_publish_destinations != durable_publish_destinations {
            return Err(RelayError::SourceAckPublishDestinationMismatch);
        }

        let ordered_ack_offsets = ordered_offsets(publish_acks)?;
        Ok(Self {
            contract: RELAY_PUBLISH_PROOF_CONTRACT,
            expected_publish_messages,
            durable_publish_acks: publish_acks.len(),
            expected_publish_destinations,
            durable_publish_destinations,
            last_publish_ack: publish_acks.last().cloned(),
            ordered_ack_offsets,
            all_publish_acks_durable: true,
        })
    }
}

pub(crate) fn durable_destinations(publish_acks: &[PublishAck]) -> Vec<PublishDestination> {
    publish_acks
        .iter()
        .map(|ack| PublishDestination {
            topic: ack.topic.clone(),
            partition: ack.partition,
        })
        .collect()
}

fn expected_destinations(messages: &[StreamMessage]) -> Result<Vec<PublishDestination>> {
    messages
        .iter()
        .map(|message| {
            Ok(PublishDestination {
                topic: message.topic.clone(),
                partition: message
                    .partition
                    .ok_or(RelayError::SourceAckPublishDestinationMissingPartition)?,
            })
        })
        .collect()
}

fn ordered_offsets(publish_acks: &[PublishAck]) -> Result<Vec<i64>> {
    let mut offsets_by_destination = BTreeMap::<(&str, i32), i64>::new();
    publish_acks
        .iter()
        .map(|ack| {
            if ack.offset < 0 {
                return Err(RelayError::PublishAckOffsetInvalid { offset: ack.offset });
            }
            if let Some(previous_offset) =
                offsets_by_destination.insert((&ack.topic, ack.partition), ack.offset)
            {
                if ack.offset <= previous_offset {
                    return Err(RelayError::PublishAckOffsetNotAdvancing {
                        topic: ack.topic.clone(),
                        partition: ack.partition,
                        previous_offset,
                        offset: ack.offset,
                    });
                }
            }
            Ok(ack.offset)
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/tests_publish_proof.rs"]
mod tests;
