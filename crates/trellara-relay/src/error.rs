use thiserror::Error;
use trellara_pg_capture::CaptureError;
use trellara_pg_extension::{NativeSourceFeedbackError, NativeWorkerSupervisionError};
use trellara_protocol::ProtocolError;
use trellara_stream::StreamError;

#[derive(Debug, Error)]
pub enum RelayError {
    #[error("capture error: {0}")]
    Capture(#[from] CaptureError),
    #[error("stream error: {0}")]
    Stream(#[from] StreamError),
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("checkpoint error: {0}")]
    Checkpoint(#[from] trellara_checkpoint::CheckpointError),
    #[error("source ACK attempted before all publish ACKs were durable: expected {expected_publish_messages}, got {durable_publish_acks}")]
    SourceAckBeforeDurablePublish {
        expected_publish_messages: usize,
        durable_publish_acks: usize,
    },
    #[error("source ACK attempted with publish ACK destinations that do not match the planned Trellara stream destinations")]
    SourceAckPublishDestinationMismatch,
    #[error("source ACK proof cannot be recorded because a planned publish message has no stream partition")]
    SourceAckPublishDestinationMissingPartition,
    #[error("publish ACK offset {offset} cannot prove durability because it is negative")]
    PublishAckOffsetInvalid { offset: i64 },
    #[error("publish ACK offset {offset} for {topic} partition {partition} did not advance beyond previous offset {previous_offset}")]
    PublishAckOffsetNotAdvancing {
        topic: String,
        partition: i32,
        previous_offset: i64,
        offset: i64,
    },
    #[error("source ACK LSN {source_ack_lsn} does not cover commit LSN {commit_lsn}")]
    SourceAckLsnBehindCommit {
        commit_lsn: String,
        source_ack_lsn: String,
    },
    #[error("loaded checkpoint {field} {value:?} cannot prove source ACK because it is ahead of {boundary_field} {boundary:?}")]
    CheckpointWatermarkInconsistent {
        field: &'static str,
        value: String,
        boundary_field: &'static str,
        boundary: String,
    },
    #[error("relay step source ACK proof {field} mismatch: step {step:?}, proof {proof:?}")]
    SourceAckProofMismatch {
        field: &'static str,
        step: String,
        proof: String,
    },
    #[error("relay step cannot become native feedback proof because {field} is not durable")]
    NativeFeedbackProofNotDurable { field: &'static str },
    #[error("native feedback rejected relay proof: {0:?}")]
    NativeFeedbackRejected(NativeSourceFeedbackError),
    #[error("native worker supervision rejected handoff: {0:?}")]
    NativeWorkerSupervisionRejected(NativeWorkerSupervisionError),
    #[error("relay stat {field} overflowed u64")]
    StatOverflow { field: &'static str },
}

pub type Result<T> = std::result::Result<T, RelayError>;
