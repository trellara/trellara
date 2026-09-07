use async_trait::async_trait;

mod rows;
use rows::{intent_from_row, receipt_from_row, status_label, to_i64, u64_to_i64};

use crate::iceberg_commit_validation::{
    ensure_same_intent, ensure_same_receipt, validate_iceberg_commit_intent,
    validate_iceberg_commit_key, validate_iceberg_commit_receipt, validate_iceberg_epoch_lookup,
};
use crate::postgres::PostgresCheckpointStore;
use crate::postgres_iceberg_commit_sql::{
    INSERT_INTENT, INSERT_RECEIPT, LIST_RECEIPTS_FOR_EPOCH, LOAD_INTENT, LOAD_RECEIPT,
};
use crate::{
    CheckpointError, IcebergCommitStore, IcebergTableCommitIntent, IcebergTableCommitKey,
    IcebergTableCommitReceipt, Result,
};

#[async_trait]
impl IcebergCommitStore for PostgresCheckpointStore {
    async fn record_iceberg_commit_intent(&self, intent: IcebergTableCommitIntent) -> Result<()> {
        validate_iceberg_commit_intent(&intent)?;
        let file_count = to_i64("file_count", intent.file_count)?;
        let record_count = u64_to_i64("record_count", intent.record_count)?;
        self.client
            .execute(
                INSERT_INTENT,
                &[
                    &intent.dataset_id,
                    &intent.epoch_id,
                    &intent.epoch_commit_id,
                    &intent.lake_table_name,
                    &intent.relation,
                    &intent.target,
                    &intent.table_commit_id,
                    &intent.manifest_digest,
                    &file_count,
                    &record_count,
                    &intent.planned_at,
                ],
            )
            .await?;
        let existing = self
            .load_iceberg_commit_intent(&intent.key())
            .await?
            .ok_or_else(|| {
                CheckpointError::Store("Iceberg intent insert was not visible".into())
            })?;
        ensure_same_intent(&existing, &intent)
    }

    async fn load_iceberg_commit_intent(
        &self,
        key: &IcebergTableCommitKey,
    ) -> Result<Option<IcebergTableCommitIntent>> {
        validate_iceberg_commit_key(key)?;
        self.client
            .query_opt(
                LOAD_INTENT,
                &[
                    &key.dataset_id,
                    &key.epoch_id,
                    &key.target,
                    &key.table_commit_id,
                ],
            )
            .await?
            .map(intent_from_row)
            .transpose()
    }

    async fn record_iceberg_commit_receipt(
        &self,
        receipt: IcebergTableCommitReceipt,
    ) -> Result<()> {
        validate_iceberg_commit_receipt(&receipt)?;
        let file_count = to_i64("file_count", receipt.file_count)?;
        let record_count = u64_to_i64("record_count", receipt.record_count)?;
        self.client
            .execute(
                INSERT_RECEIPT,
                &[
                    &receipt.dataset_id,
                    &receipt.epoch_id,
                    &receipt.epoch_commit_id,
                    &receipt.target,
                    &receipt.table_commit_id,
                    &receipt.snapshot_id,
                    &file_count,
                    &record_count,
                    &status_label(receipt.status),
                    &receipt.committed_at,
                ],
            )
            .await?;
        let existing = self
            .load_iceberg_commit_receipt(&receipt.key())
            .await?
            .ok_or_else(|| {
                CheckpointError::Store("Iceberg receipt insert was not visible".into())
            })?;
        ensure_same_receipt(&existing, &receipt)
    }

    async fn load_iceberg_commit_receipt(
        &self,
        key: &IcebergTableCommitKey,
    ) -> Result<Option<IcebergTableCommitReceipt>> {
        validate_iceberg_commit_key(key)?;
        self.client
            .query_opt(
                LOAD_RECEIPT,
                &[
                    &key.dataset_id,
                    &key.epoch_id,
                    &key.target,
                    &key.table_commit_id,
                ],
            )
            .await?
            .map(receipt_from_row)
            .transpose()
    }

    async fn list_iceberg_commit_receipts_for_epoch(
        &self,
        dataset_id: &str,
        epoch_id: &str,
    ) -> Result<Vec<IcebergTableCommitReceipt>> {
        validate_iceberg_epoch_lookup(dataset_id, epoch_id)?;
        self.client
            .query(LIST_RECEIPTS_FOR_EPOCH, &[&dataset_id, &epoch_id])
            .await?
            .into_iter()
            .map(receipt_from_row)
            .collect()
    }
}
