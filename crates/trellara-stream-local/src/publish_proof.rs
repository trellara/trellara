use std::path::Path;

use trellara_stream::PublishAck;

use crate::index::recover_topic_index;
use crate::index_read::read_record_at;
use crate::inspection::inspect_local_stream;
use crate::paths::{topic_index_path, topic_path};
use crate::{validate_topic, LocalDurability, LocalStreamError, Result};

pub const LOCAL_PUBLISH_ACK_PROOF_CONTRACT: &str =
    "local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalPublishAckProof {
    pub contract: &'static str,
    pub durability: &'static str,
    pub crash_safe_ack: bool,
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub key: String,
    pub indexed: bool,
    pub replayable: bool,
    pub index_status: &'static str,
    pub torn_tail_bytes: u64,
}

pub fn local_publish_ack_proof(
    root: impl AsRef<Path>,
    ack: &PublishAck,
) -> Result<LocalPublishAckProof> {
    local_publish_ack_proof_with_durability(root, ack, LocalDurability::default())
}

pub fn local_publish_ack_proof_with_durability(
    root: impl AsRef<Path>,
    ack: &PublishAck,
    durability: LocalDurability,
) -> Result<LocalPublishAckProof> {
    validate_topic(&ack.topic)?;
    if ack.partition != 0 {
        return Err(LocalStreamError::InvalidPublishAckProof {
            topic: ack.topic.clone(),
            offset: ack.offset,
            reason: "local stream publish ACK partition must be 0".to_string(),
        });
    }
    if ack.offset < 0 {
        return Err(LocalStreamError::NegativeCursorOffset { offset: ack.offset });
    }

    let root = root.as_ref();
    let topic_path = topic_path(root, &ack.topic);
    let index_path = topic_index_path(root, &ack.topic);
    let state = recover_topic_index(&topic_path, &index_path, durability)?;
    let indexed = usize::try_from(ack.offset)
        .ok()
        .is_some_and(|offset| state.positions.get(offset).is_some());
    if !indexed {
        return Err(LocalStreamError::InvalidPublishAckProof {
            topic: ack.topic.clone(),
            offset: ack.offset,
            reason: "publish ACK offset is not present in the recovered topic index".to_string(),
        });
    }

    let Some(message) =
        read_record_at(&topic_path, &index_path, &ack.topic, ack.offset, durability)?
    else {
        return Err(LocalStreamError::InvalidPublishAckProof {
            topic: ack.topic.clone(),
            offset: ack.offset,
            reason: "publish ACK offset is not replayable".to_string(),
        });
    };
    let topic = inspect_local_stream(root)?
        .topics
        .into_iter()
        .find(|candidate| candidate.topic == ack.topic)
        .ok_or_else(|| LocalStreamError::InvalidPublishAckProof {
            topic: ack.topic.clone(),
            offset: ack.offset,
            reason: "publish ACK topic is missing from local stream inspection".to_string(),
        })?;
    if topic.torn_tail_bytes != 0 || !topic.replayable {
        return Err(LocalStreamError::InvalidPublishAckProof {
            topic: ack.topic.clone(),
            offset: ack.offset,
            reason: "publish ACK topic is not fully replayable without a torn tail".to_string(),
        });
    }

    Ok(LocalPublishAckProof {
        contract: LOCAL_PUBLISH_ACK_PROOF_CONTRACT,
        durability: durability.as_str(),
        crash_safe_ack: durability.crash_safe_ack(),
        topic: ack.topic.clone(),
        partition: ack.partition,
        offset: ack.offset,
        key: message.key,
        indexed,
        replayable: topic.replayable,
        index_status: topic.index_status.as_str(),
        torn_tail_bytes: topic.torn_tail_bytes,
    })
}
