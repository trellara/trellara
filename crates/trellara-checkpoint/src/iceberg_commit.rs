use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IcebergTableCommitKey {
    pub dataset_id: String,
    pub epoch_id: String,
    pub target: String,
    pub table_commit_id: String,
}

impl IcebergTableCommitKey {
    pub fn new(
        dataset_id: impl Into<String>,
        epoch_id: impl Into<String>,
        target: impl Into<String>,
        table_commit_id: impl Into<String>,
    ) -> Self {
        Self {
            dataset_id: dataset_id.into(),
            epoch_id: epoch_id.into(),
            target: target.into(),
            table_commit_id: table_commit_id.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableCommitIntent {
    pub dataset_id: String,
    pub epoch_id: String,
    pub epoch_commit_id: String,
    pub lake_table_name: String,
    pub relation: String,
    pub target: String,
    pub table_commit_id: String,
    pub manifest_digest: String,
    pub file_count: usize,
    pub record_count: u64,
    pub planned_at: String,
}

impl IcebergTableCommitIntent {
    #[must_use]
    pub fn key(&self) -> IcebergTableCommitKey {
        IcebergTableCommitKey::new(
            &self.dataset_id,
            &self.epoch_id,
            &self.target,
            &self.table_commit_id,
        )
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergTableCommitStatus {
    Committed,
    AlreadyCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableCommitReceipt {
    pub dataset_id: String,
    pub epoch_id: String,
    pub epoch_commit_id: String,
    pub target: String,
    pub table_commit_id: String,
    pub snapshot_id: i64,
    pub file_count: usize,
    pub record_count: u64,
    pub status: IcebergTableCommitStatus,
    pub committed_at: String,
}

impl IcebergTableCommitReceipt {
    #[must_use]
    pub fn key(&self) -> IcebergTableCommitKey {
        IcebergTableCommitKey::new(
            &self.dataset_id,
            &self.epoch_id,
            &self.target,
            &self.table_commit_id,
        )
    }
}

#[async_trait]
pub trait IcebergCommitStore: Send + Sync {
    async fn record_iceberg_commit_intent(&self, intent: IcebergTableCommitIntent) -> Result<()>;

    async fn load_iceberg_commit_intent(
        &self,
        key: &IcebergTableCommitKey,
    ) -> Result<Option<IcebergTableCommitIntent>>;

    async fn record_iceberg_commit_receipt(&self, receipt: IcebergTableCommitReceipt)
        -> Result<()>;

    async fn load_iceberg_commit_receipt(
        &self,
        key: &IcebergTableCommitKey,
    ) -> Result<Option<IcebergTableCommitReceipt>>;

    async fn list_iceberg_commit_receipts_for_epoch(
        &self,
        dataset_id: &str,
        epoch_id: &str,
    ) -> Result<Vec<IcebergTableCommitReceipt>>;
}
