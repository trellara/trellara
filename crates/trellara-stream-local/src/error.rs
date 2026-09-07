use std::io;

use thiserror::Error;
use trellara_protocol::{BarrierHoldReason, ProtocolError};
use trellara_stream::StreamError;

#[derive(Debug, Error)]
pub enum LocalStreamError {
    #[error("local stream I/O error at {path}: {source}")]
    Io { path: String, source: io::Error },
    #[error("invalid local stream topic {topic:?}")]
    InvalidTopic { topic: String },
    #[error("local consumer requires at least one topic")]
    MissingTopics,
    #[error("cannot ack message without local stream position")]
    MissingPosition,
    #[error("cannot ack topic {topic:?}; consumer is subscribed to {topics:?}")]
    UnknownAckTopic { topic: String, topics: Vec<String> },
    #[error(
        "cannot ack local topic {topic:?} partition {partition}; local streams use partition 0"
    )]
    UnsupportedAckPartition { topic: String, partition: i32 },
    #[error("local cursor offset must be non-negative, got {offset}")]
    NegativeCursorOffset { offset: i64 },
    #[error("local cursor offset {offset} is ahead of topic {topic:?} depth {message_count}; pass --allow-ahead for an intentional fast-forward")]
    CursorOffsetAhead {
        topic: String,
        offset: i64,
        message_count: i64,
    },
    #[error("cannot ack local topic {topic:?} offset {offset}; durable cursor is at next_offset {current_next_offset}")]
    NonContiguousAck {
        topic: String,
        offset: i64,
        current_next_offset: i64,
    },
    #[error("local cursor offset overflow after acking {offset}")]
    CursorOffsetOverflow { offset: i64 },
    #[error("invalid local cursor at {path}: {value:?}")]
    InvalidCursor { path: String, value: String },
    #[error("local stream frame in {path} is corrupt")]
    CorruptFrame { path: String },
    #[error("local publish ACK proof failed for topic {topic:?} offset {offset}: {reason}")]
    InvalidPublishAckProof {
        topic: String,
        offset: i64,
        reason: String,
    },
    #[error("local source ACK durability proof failed: {reason}")]
    InvalidSourceAckDurabilityProof { reason: String },
    #[error("record field {field} is too large for local stream frame")]
    FieldTooLarge { field: &'static str },
    #[error("local stream message field {field} is invalid: {reason}")]
    InvalidMessageField { field: &'static str, reason: String },
    #[error("local stream path {path} has no parent directory")]
    MissingParentPath { path: String },
    #[error("missing local barrier message at topic {topic:?} offset {offset}")]
    MissingBarrierMessage { topic: String, offset: i64 },
    #[error("failed to decode {message_kind} at topic {topic:?} offset {offset}: {source}")]
    BarrierDecode {
        message_kind: &'static str,
        topic: String,
        offset: i64,
        source: prost::DecodeError,
    },
    #[error("partition chunk at topic {topic:?} offset {offset} decoded as partition {actual_partition_id}, expected partition {expected_partition_id}")]
    BarrierPartitionMismatch {
        topic: String,
        offset: i64,
        expected_partition_id: u32,
        actual_partition_id: u32,
    },
    #[error("local barrier reconstruction request field {field} is invalid: {reason}")]
    InvalidBarrierReconstructionField { field: &'static str, reason: String },
    #[error("local barrier reconstruction offset {field} must be non-negative, got {offset}")]
    NegativeBarrierOffset { field: &'static str, offset: i64 },
    #[error("local barrier reconstruction request repeats partition {partition_id}")]
    DuplicateBarrierPartitionOffset { partition_id: u32 },
    #[error("local barrier header {field} {header:?} does not match payload {payload:?} at topic {topic:?} offset {offset}")]
    BarrierHeaderPayloadMismatch {
        topic: String,
        offset: i64,
        field: &'static str,
        header: String,
        payload: String,
    },
    #[error(
        "local barrier header {field} appears more than once at topic {topic:?} offset {offset}"
    )]
    DuplicateBarrierHeader {
        topic: String,
        offset: i64,
        field: &'static str,
    },
    #[error(
        "local barrier transaction {transaction_id} is held: {reason:?}; missing partitions {missing_partitions:?}"
    )]
    BarrierTransactionHeld {
        transaction_id: String,
        reason: BarrierHoldReason,
        missing_partitions: Vec<u32>,
    },
    #[error("local barrier reconstruction failed: {0}")]
    BarrierProtocol(#[from] ProtocolError),
}

impl From<LocalStreamError> for StreamError {
    fn from(error: LocalStreamError) -> Self {
        match error {
            LocalStreamError::Io { .. }
            | LocalStreamError::InvalidTopic { .. }
            | LocalStreamError::CorruptFrame { .. }
            | LocalStreamError::InvalidPublishAckProof { .. }
            | LocalStreamError::InvalidSourceAckDurabilityProof { .. }
            | LocalStreamError::FieldTooLarge { .. }
            | LocalStreamError::InvalidMessageField { .. }
            | LocalStreamError::MissingParentPath { .. }
            | LocalStreamError::MissingBarrierMessage { .. }
            | LocalStreamError::BarrierDecode { .. }
            | LocalStreamError::BarrierPartitionMismatch { .. }
            | LocalStreamError::InvalidBarrierReconstructionField { .. }
            | LocalStreamError::NegativeBarrierOffset { .. }
            | LocalStreamError::DuplicateBarrierPartitionOffset { .. }
            | LocalStreamError::BarrierHeaderPayloadMismatch { .. }
            | LocalStreamError::DuplicateBarrierHeader { .. }
            | LocalStreamError::BarrierTransactionHeld { .. }
            | LocalStreamError::BarrierProtocol { .. } => StreamError::Publisher(error.to_string()),
            LocalStreamError::MissingTopics
            | LocalStreamError::MissingPosition
            | LocalStreamError::InvalidCursor { .. }
            | LocalStreamError::NegativeCursorOffset { .. }
            | LocalStreamError::CursorOffsetAhead { .. }
            | LocalStreamError::NonContiguousAck { .. }
            | LocalStreamError::CursorOffsetOverflow { .. }
            | LocalStreamError::UnsupportedAckPartition { .. }
            | LocalStreamError::UnknownAckTopic { .. } => StreamError::Consumer(error.to_string()),
        }
    }
}
