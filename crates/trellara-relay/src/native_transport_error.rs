use std::num::TryFromIntError;
use std::path::PathBuf;

use thiserror::Error;
use trellara_pg_extension::NativeRelayWireError;

#[derive(Debug, Error)]
pub enum NativeRelayTransportError {
    #[error("native relay secret cannot be empty")]
    EmptySecret,
    #[error("native relay I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("native relay wire validation failed: {0}")]
    Wire(#[from] NativeRelayWireError),
    #[error("native relay integer conversion failed: {0}")]
    Integer(#[from] TryFromIntError),
    #[error("native relay message is {0} bytes, outside the allowed bound")]
    MessageTooLarge(usize),
    #[error("durable native frame at commit LSN {commit_lsn} conflicts with prior evidence")]
    ConflictingDurableFrame { commit_lsn: u64 },
    #[error("Kafka proof at commit LSN {commit_lsn} conflicts with prior durable evidence")]
    ConflictingKafkaProof { commit_lsn: u64 },
    #[error("Kafka publish proof is corrupt: {0}")]
    PublishProofCorrupt(&'static str),
    #[error("Kafka configuration failed: {0}")]
    KafkaConfiguration(String),
    #[error("Kafka publication failed: {0}")]
    KafkaPublish(String),
    #[error("Kafka acknowledged a destination other than the planned topic and partition")]
    KafkaDestinationMismatch,
    #[error("Kafka publish proof does not match the native transaction frame")]
    KafkaProofMismatch,
    #[error("Kafka publish acknowledgement returned invalid offset {0}")]
    InvalidKafkaOffset(i64),
    #[error("refusing to replace non-socket path {0}")]
    UnsafeSocketPath(PathBuf),
}

pub type NativeRelayTransportResult<T> = Result<T, NativeRelayTransportError>;
