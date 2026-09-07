use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use trellara_protocol::{Checkpoint, TransactionEnvelope};

use crate::checkpoint_rewind::reject_checkpoint_rewind;
use crate::checkpoint_validation::{validate_checkpoint, validate_flow_key};
use crate::ddl_barrier_identity::{DdlBarrierAckKey, DdlBarrierKey};
use crate::iceberg_commit::IcebergTableCommitKey;
use crate::lsn::merge_checkpoint;
use crate::snapshot::{SnapshotRun, SnapshotTableProgress};
use crate::transaction_key_validation::validate_transaction_key;
use crate::{
    ApplyDecision, CheckpointStore, DdlBarrier, DdlBarrierAck, DedupStore, FlowKey,
    IcebergTableCommitIntent, IcebergTableCommitReceipt, Result, TransactionKey,
};

pub(crate) type SnapshotRunKey = (FlowKey, String);
pub(crate) type SnapshotTableProgressKey = (FlowKey, String, String);
type SnapshotRunMap = HashMap<SnapshotRunKey, SnapshotRun>;
type SnapshotTableProgressMap = HashMap<SnapshotTableProgressKey, SnapshotTableProgress>;

#[derive(Clone, Default)]
pub struct InMemoryCheckpointStore {
    checkpoints: Arc<RwLock<HashMap<FlowKey, Checkpoint>>>,
    applied_transactions: Arc<RwLock<HashSet<TransactionKey>>>,
    pub(crate) ddl_barriers: Arc<RwLock<HashMap<DdlBarrierKey, DdlBarrier>>>,
    pub(crate) ddl_barrier_acks: Arc<RwLock<HashMap<DdlBarrierAckKey, DdlBarrierAck>>>,
    pub(crate) iceberg_commit_intents:
        Arc<RwLock<HashMap<IcebergTableCommitKey, IcebergTableCommitIntent>>>,
    pub(crate) iceberg_commit_receipts:
        Arc<RwLock<HashMap<IcebergTableCommitKey, IcebergTableCommitReceipt>>>,
    pub(crate) snapshot_runs: Arc<RwLock<SnapshotRunMap>>,
    pub(crate) snapshot_table_progress: Arc<RwLock<SnapshotTableProgressMap>>,
}

impl InMemoryCheckpointStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record_envelope_applied(&self, envelope: &TransactionEnvelope) -> Result<()> {
        let transaction_key = TransactionKey::try_from_envelope(envelope)?;
        self.record_applied(transaction_key.clone()).await?;
        self.save_checkpoint(Checkpoint {
            source_id: transaction_key.source_id,
            dataset_id: transaction_key.dataset_id,
            last_seen_lsn: transaction_key.commit_lsn.clone(),
            last_durable_lsn: transaction_key.commit_lsn.clone(),
            last_applied_lsn: transaction_key.commit_lsn,
        })
        .await
    }
}

#[async_trait]
impl CheckpointStore for InMemoryCheckpointStore {
    async fn load_checkpoint(&self, flow: &FlowKey) -> Result<Option<Checkpoint>> {
        validate_flow_key(flow)?;
        Ok(self.checkpoints.read().await.get(flow).cloned())
    }

    async fn save_checkpoint(&self, checkpoint: Checkpoint) -> Result<()> {
        validate_checkpoint(&checkpoint)?;
        let key = FlowKey::new(&checkpoint.source_id, &checkpoint.dataset_id);
        let mut checkpoints = self.checkpoints.write().await;
        let checkpoint = checkpoints
            .get(&key)
            .map(|existing| {
                reject_checkpoint_rewind(existing, &checkpoint)?;
                Ok::<Checkpoint, crate::CheckpointError>(merge_checkpoint(
                    existing,
                    checkpoint.clone(),
                ))
            })
            .transpose()?
            .unwrap_or(checkpoint);
        checkpoints.insert(key, checkpoint);
        Ok(())
    }
}

#[async_trait]
impl DedupStore for InMemoryCheckpointStore {
    async fn apply_decision(&self, transaction: &TransactionKey) -> Result<ApplyDecision> {
        validate_transaction_key(transaction)?;
        if self.applied_transactions.read().await.contains(transaction) {
            Ok(ApplyDecision::SkipDuplicate)
        } else {
            Ok(ApplyDecision::Apply)
        }
    }

    async fn record_applied(&self, transaction: TransactionKey) -> Result<()> {
        validate_transaction_key(&transaction)?;
        self.applied_transactions.write().await.insert(transaction);
        Ok(())
    }
}
