use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use trellara_protocol::{Checkpoint, ProtocolError, TransactionBoundaryKey, TransactionEnvelope};

use crate::lsn::parse_lsn;
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FlowKey {
    pub source_id: String,
    pub dataset_id: String,
}

impl FlowKey {
    pub fn new(source_id: impl Into<String>, dataset_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            dataset_id: dataset_id.into(),
        }
    }
}

impl From<&TransactionEnvelope> for FlowKey {
    fn from(envelope: &TransactionEnvelope) -> Self {
        Self::new(&envelope.source_id, &envelope.dataset_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TransactionKey {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub transaction_id: String,
    pub commit_lsn: String,
}

impl TransactionKey {
    pub fn from_envelope(envelope: &TransactionEnvelope) -> Self {
        Self {
            source_id: envelope.source_id.clone(),
            database_id: envelope.database_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            transaction_id: envelope.transaction_id.clone(),
            commit_lsn: envelope.commit_lsn.clone(),
        }
    }

    pub fn try_from_envelope(envelope: &TransactionEnvelope) -> Result<Self> {
        let key = Self::from_envelope(envelope);
        Ok(Self::from_boundary_key(key.to_boundary_key()?))
    }

    pub fn from_boundary_key(boundary: TransactionBoundaryKey) -> Self {
        Self {
            source_id: boundary.source_id,
            database_id: boundary.database_id,
            dataset_id: boundary.dataset_id,
            transaction_id: boundary.transaction_id,
            commit_lsn: boundary.commit_lsn,
        }
    }

    pub fn to_boundary_key(&self) -> Result<TransactionBoundaryKey> {
        TransactionBoundaryKey::new_with_database(
            self.source_id.clone(),
            self.database_id.clone(),
            self.dataset_id.clone(),
            self.transaction_id.clone(),
            self.commit_lsn.clone(),
        )
        .map_err(transaction_boundary_error)
    }

    pub fn validate(&self) -> Result<()> {
        self.to_boundary_key().map(|_| ())
    }
}

impl From<ProtocolError> for crate::CheckpointError {
    fn from(error: ProtocolError) -> Self {
        Self::Store(format!("transaction boundary identity is invalid: {error}"))
    }
}

fn transaction_boundary_error(error: ProtocolError) -> crate::CheckpointError {
    match &error {
        ProtocolError::InvalidLsn { .. } => crate::CheckpointError::Store(format!(
            "transaction key commit_lsn boundary is invalid: {error}"
        )),
        _ => crate::CheckpointError::Store(format!(
            "transaction boundary identity is invalid: {error}"
        )),
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointLag {
    pub source_id: String,
    pub dataset_id: String,
    pub last_seen_lsn: String,
    pub last_durable_lsn: String,
    pub last_applied_lsn: String,
    pub seen_to_durable_bytes: u64,
    pub durable_to_applied_bytes: u64,
    pub seen_to_applied_bytes: u64,
    pub source_is_durable: bool,
    pub target_is_caught_up: bool,
}

impl CheckpointLag {
    pub fn from_checkpoint(checkpoint: &Checkpoint) -> Self {
        let last_seen = parse_lsn(&checkpoint.last_seen_lsn);
        let last_durable = parse_lsn(&checkpoint.last_durable_lsn);
        let last_applied = parse_lsn(&checkpoint.last_applied_lsn);

        Self {
            source_id: checkpoint.source_id.clone(),
            dataset_id: checkpoint.dataset_id.clone(),
            last_seen_lsn: checkpoint.last_seen_lsn.clone(),
            last_durable_lsn: checkpoint.last_durable_lsn.clone(),
            last_applied_lsn: checkpoint.last_applied_lsn.clone(),
            seen_to_durable_bytes: last_seen.saturating_sub(last_durable),
            durable_to_applied_bytes: last_durable.saturating_sub(last_applied),
            seen_to_applied_bytes: last_seen.saturating_sub(last_applied),
            source_is_durable: last_durable >= last_seen,
            target_is_caught_up: last_applied >= last_durable,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplyDecision {
    Apply,
    SkipDuplicate,
}

#[async_trait]
pub trait CheckpointStore: Send + Sync {
    async fn load_checkpoint(&self, flow: &FlowKey) -> Result<Option<Checkpoint>>;
    async fn save_checkpoint(&self, checkpoint: Checkpoint) -> Result<()>;
}

#[async_trait]
pub trait DedupStore: Send + Sync {
    async fn apply_decision(&self, transaction: &TransactionKey) -> Result<ApplyDecision>;
    async fn record_applied(&self, transaction: TransactionKey) -> Result<()>;
}
