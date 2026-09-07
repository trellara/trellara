use std::path::Path;

use async_trait::async_trait;

mod barrier_header_payload;
mod barrier_headers;
mod barrier_readers;
mod barrier_reconstruct;
mod barrier_reconstruct_request;
mod barrier_topics;
mod consumer;
mod consumer_ack;
mod cursor;
mod cursor_storage;
mod error;
mod frame;
mod frame_body;
mod frame_checksum;
mod frame_headers;
mod frame_io;
mod frame_limits;
mod index;
mod index_entries;
mod index_file;
mod index_read;
mod index_scan;
mod inspection;
mod inspection_cursors;
mod inspection_topics;
mod inspection_types;
mod paths;
mod publish_proof;
mod publisher;
mod recovery_actions;
mod recovery_inspection;
mod source_ack_proof;
mod topic;
mod types;

use trellara_stream::{
    PublishAck, StreamConsumer, StreamError, StreamMessage, StreamPosition, StreamPublisher,
};

use index_read::read_record_at;
use paths::{topic_index_path, topic_path};
use publisher::publish_local;
use topic::validate_topic;

pub use barrier_reconstruct::{reconstruct_local_barrier_transaction, LocalBarrierTransaction};
pub use barrier_reconstruct_request::{
    LocalBarrierReconstructionRequest, LocalPartitionChunkOffset,
};
pub use cursor::{set_local_cursor, set_local_cursor_with_policy};
pub use inspection::inspect_local_stream;
pub use inspection_types::{
    LocalCursorInspection, LocalCursorStatus, LocalStreamInspection, LocalTopicIndexStatus,
    LocalTopicInspection, LocalTopicProofHeader,
};
pub use publish_proof::{
    local_publish_ack_proof, local_publish_ack_proof_with_durability, LocalPublishAckProof,
    LOCAL_PUBLISH_ACK_PROOF_CONTRACT,
};
pub use recovery_inspection::{
    inspect_local_recovery, LocalRecoveryCursorBlocker, LocalRecoveryInspection,
    LocalRecoveryTopicEvidence,
};
pub use source_ack_proof::{
    local_source_ack_durability_proof, local_source_ack_durability_proof_with_durability,
    LocalSourceAckDurabilityProof, LOCAL_SOURCE_ACK_DURABILITY_CONTRACT,
};

pub(crate) use consumer::next_local;
pub(crate) use consumer_ack::ack_local;
pub use consumer_ack::LocalAckOutcome;
pub(crate) use cursor_storage::sync_parent_dir;
pub use error::LocalStreamError;
pub(crate) use types::LocalReadCursor;
pub use types::{
    LocalConsumer, LocalConsumerConfig, LocalDurability, LocalPublisher, LocalPublisherConfig,
};
pub type Result<T> = std::result::Result<T, LocalStreamError>;

#[cfg(test)]
use cursor_storage::read_cursor;
#[cfg(test)]
use index::write_index;
#[cfg(test)]
use paths::topic_dir;
#[cfg(test)]
use std::fs::{self, OpenOptions};

pub fn read_local_message_at(
    root: impl AsRef<Path>,
    topic: impl AsRef<str>,
    offset: i64,
) -> Result<Option<StreamMessage>> {
    let root = root.as_ref();
    let topic = topic.as_ref();
    validate_topic(topic)?;
    let mut message = read_record_at(
        &topic_path(root, topic),
        &topic_index_path(root, topic),
        topic,
        offset,
        LocalDurability::default(),
    )?;
    if let Some(message) = &mut message {
        message.position = Some(StreamPosition {
            topic: topic.to_string(),
            partition: 0,
            offset,
        });
        message.partition = Some(0);
    }
    Ok(message)
}

#[async_trait]
impl StreamPublisher for LocalPublisher {
    async fn publish(
        &self,
        message: StreamMessage,
    ) -> std::result::Result<PublishAck, StreamError> {
        publish_local(&self.config, message).map_err(StreamError::from)
    }
}

#[async_trait]
impl StreamConsumer for LocalConsumer {
    async fn next(&mut self) -> std::result::Result<Option<StreamMessage>, StreamError> {
        next_local(self).map_err(StreamError::from)
    }

    async fn ack(&mut self, message: &StreamMessage) -> std::result::Result<(), StreamError> {
        self.ack_with_outcome(message).map_err(StreamError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
