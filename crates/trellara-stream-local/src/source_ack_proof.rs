use std::collections::BTreeMap;
use std::path::Path;

use trellara_stream::{PublishAck, StreamMessage};

use crate::publish_proof::{local_publish_ack_proof_with_durability, LocalPublishAckProof};
use crate::{LocalDurability, LocalStreamError, Result};

pub const LOCAL_SOURCE_ACK_DURABILITY_CONTRACT: &str =
    "every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalSourceAckDurabilityProof {
    pub contract: &'static str,
    pub durability: &'static str,
    pub crash_safe_ack: bool,
    pub expected_publish_messages: usize,
    pub durable_publish_acks: usize,
    pub all_publish_acks_proven: bool,
    pub ack_proofs: Vec<LocalPublishAckProof>,
}

pub fn local_source_ack_durability_proof(
    root: impl AsRef<Path>,
    messages: &[StreamMessage],
    publish_acks: &[PublishAck],
    expected_publish_messages: usize,
) -> Result<LocalSourceAckDurabilityProof> {
    local_source_ack_durability_proof_with_durability(
        root,
        messages,
        publish_acks,
        expected_publish_messages,
        LocalDurability::default(),
    )
}

pub fn local_source_ack_durability_proof_with_durability(
    root: impl AsRef<Path>,
    messages: &[StreamMessage],
    publish_acks: &[PublishAck],
    expected_publish_messages: usize,
    durability: LocalDurability,
) -> Result<LocalSourceAckDurabilityProof> {
    if messages.len() != expected_publish_messages {
        return invalid_source_ack_proof(format!(
            "expected {expected_publish_messages} publish messages but got {}",
            messages.len()
        ));
    }
    if publish_acks.len() != expected_publish_messages {
        return invalid_source_ack_proof(format!(
            "expected {expected_publish_messages} durable publish ACKs but got {}",
            publish_acks.len()
        ));
    }

    let root = root.as_ref();
    let mut previous_offsets = BTreeMap::<&str, i64>::new();
    let mut ack_proofs = Vec::with_capacity(publish_acks.len());
    for (message, ack) in messages.iter().zip(publish_acks) {
        validate_ack_matches_message(message, ack)?;
        validate_ack_advances(&mut previous_offsets, ack)?;
        let proof = local_publish_ack_proof_with_durability(root, ack, durability)?;
        if proof.key != message.key {
            return invalid_source_ack_proof(format!(
                "publish ACK at topic {:?} offset {} replays key {:?}, expected {:?}",
                ack.topic, ack.offset, proof.key, message.key
            ));
        }
        ack_proofs.push(proof);
    }

    Ok(LocalSourceAckDurabilityProof {
        contract: LOCAL_SOURCE_ACK_DURABILITY_CONTRACT,
        durability: durability.as_str(),
        crash_safe_ack: durability.crash_safe_ack(),
        expected_publish_messages,
        durable_publish_acks: publish_acks.len(),
        all_publish_acks_proven: true,
        ack_proofs,
    })
}

fn validate_ack_matches_message(message: &StreamMessage, ack: &PublishAck) -> Result<()> {
    if ack.topic != message.topic {
        return invalid_source_ack_proof(format!(
            "publish ACK topic {:?} does not match message topic {:?}",
            ack.topic, message.topic
        ));
    }
    if ack.partition != 0 {
        return invalid_source_ack_proof(format!(
            "local publish ACK partition must be 0, got {}",
            ack.partition
        ));
    }
    Ok(())
}

fn validate_ack_advances<'a>(
    previous_offsets: &mut BTreeMap<&'a str, i64>,
    ack: &'a PublishAck,
) -> Result<()> {
    if let Some(previous_offset) = previous_offsets.insert(&ack.topic, ack.offset) {
        if ack.offset <= previous_offset {
            return invalid_source_ack_proof(format!(
                "publish ACK offset {} does not advance previous offset {} for topic {:?}",
                ack.offset, previous_offset, ack.topic
            ));
        }
    }
    Ok(())
}

fn invalid_source_ack_proof<T>(reason: String) -> Result<T> {
    Err(LocalStreamError::InvalidSourceAckDurabilityProof { reason })
}
