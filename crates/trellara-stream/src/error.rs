use thiserror::Error;
use trellara_protocol::ProtocolError;

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("invalid topic component {component:?}: {reason}")]
    InvalidTopicComponent { component: String, reason: String },
    #[error("barrier payload {field} mismatch: envelope {envelope:?}, payload {payload:?}")]
    BarrierPayloadMismatch {
        field: &'static str,
        envelope: String,
        payload: String,
    },
    #[error("stream message field {field} is invalid: {reason}")]
    InvalidMessageField { field: &'static str, reason: String },
    #[error("partition id {partition_id} exceeds supported stream partition {max_supported}")]
    InvalidStreamPartition {
        partition_id: u32,
        max_supported: u32,
    },
    #[error("publisher error: {0}")]
    Publisher(String),
    #[error("consumer error: {0}")]
    Consumer(String),
}
