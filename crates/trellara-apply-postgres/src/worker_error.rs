use thiserror::Error;
use trellara_protocol::{BarrierHoldReason, ProtocolError};
use trellara_stream::StreamError;

use crate::ApplyError;

#[derive(Debug, Error)]
pub enum ApplyWorkerError {
    #[error("stream error: {0}")]
    Stream(#[from] StreamError),
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("decode error: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("apply error: {0}")]
    Apply(#[from] ApplyError),
    #[error("message is missing required header {0}")]
    MissingHeader(&'static str),
    #[error("stream header {key} is invalid: {reason}")]
    InvalidHeaderField { key: &'static str, reason: String },
    #[error("unsupported stream message kind {0}")]
    UnsupportedMessageKind(String),
    #[error("duplicate manifest has conflicting contents for transaction {transaction_id}")]
    DuplicateManifest { transaction_id: String },
    #[error("commit marker does not match manifest for transaction {transaction_id}")]
    CommitMarkerMismatch { transaction_id: String },
    #[error("ready barrier transaction {transaction_key} is missing from pending state")]
    ReadyTransactionMissing { transaction_key: String },
    #[error("ready barrier transaction {transaction_key} is missing manifest evidence")]
    ReadyTransactionMissingManifest { transaction_key: String },
    #[error("ready barrier transaction {transaction_key} is missing commit marker evidence")]
    ReadyTransactionMissingCommitMarker { transaction_key: String },
    #[error(
        "ready barrier transaction {transaction_id} is missing partition chunk {partition_id}"
    )]
    ReadyTransactionMissingChunk {
        transaction_id: String,
        partition_id: u32,
    },
    #[error(
        "barrier transaction {transaction_id} is not ready: {reason:?}; missing partitions {missing_partitions:?}"
    )]
    BarrierTransactionHeld {
        transaction_id: String,
        reason: BarrierHoldReason,
        missing_partitions: Vec<u32>,
    },
    #[error("apply worker stat {field} overflowed u64")]
    StatOverflow { field: &'static str },
    #[error(
        "apply outcome commit_lsn {actual:?} does not match envelope commit_lsn {expected:?} for transaction {transaction_id}"
    )]
    ApplyOutcomeCommitLsnMismatch {
        transaction_id: String,
        expected: String,
        actual: String,
    },
    #[error("stream header {field} {header:?} does not match payload {payload:?}")]
    HeaderPayloadMismatch {
        field: &'static str,
        header: String,
        payload: String,
    },
}

pub type ApplyWorkerResult<T> = std::result::Result<T, ApplyWorkerError>;
