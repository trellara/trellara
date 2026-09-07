use async_trait::async_trait;

use crate::iceberg_commit_validation::{
    ensure_same_intent, ensure_same_receipt, validate_iceberg_commit_intent,
    validate_iceberg_commit_key, validate_iceberg_commit_receipt, validate_iceberg_epoch_lookup,
};
use crate::{
    IcebergCommitStore, IcebergTableCommitIntent, IcebergTableCommitKey, IcebergTableCommitReceipt,
    InMemoryCheckpointStore, Result,
};

#[async_trait]
impl IcebergCommitStore for InMemoryCheckpointStore {
    async fn record_iceberg_commit_intent(&self, intent: IcebergTableCommitIntent) -> Result<()> {
        validate_iceberg_commit_intent(&intent)?;
        let key = intent.key();
        let mut intents = self.iceberg_commit_intents.write().await;
        if let Some(existing) = intents.get(&key) {
            ensure_same_intent(existing, &intent)?;
            return Ok(());
        }
        intents.insert(key, intent);
        Ok(())
    }

    async fn load_iceberg_commit_intent(
        &self,
        key: &IcebergTableCommitKey,
    ) -> Result<Option<IcebergTableCommitIntent>> {
        validate_iceberg_commit_key(key)?;
        Ok(self.iceberg_commit_intents.read().await.get(key).cloned())
    }

    async fn record_iceberg_commit_receipt(
        &self,
        receipt: IcebergTableCommitReceipt,
    ) -> Result<()> {
        validate_iceberg_commit_receipt(&receipt)?;
        let key = receipt.key();
        let mut receipts = self.iceberg_commit_receipts.write().await;
        if let Some(existing) = receipts.get(&key) {
            ensure_same_receipt(existing, &receipt)?;
            return Ok(());
        }
        receipts.insert(key, receipt);
        Ok(())
    }

    async fn load_iceberg_commit_receipt(
        &self,
        key: &IcebergTableCommitKey,
    ) -> Result<Option<IcebergTableCommitReceipt>> {
        validate_iceberg_commit_key(key)?;
        Ok(self.iceberg_commit_receipts.read().await.get(key).cloned())
    }

    async fn list_iceberg_commit_receipts_for_epoch(
        &self,
        dataset_id: &str,
        epoch_id: &str,
    ) -> Result<Vec<IcebergTableCommitReceipt>> {
        validate_iceberg_epoch_lookup(dataset_id, epoch_id)?;
        let mut receipts = self
            .iceberg_commit_receipts
            .read()
            .await
            .values()
            .filter(|receipt| receipt.dataset_id == dataset_id && receipt.epoch_id == epoch_id)
            .cloned()
            .collect::<Vec<_>>();
        receipts.sort_by(|left, right| {
            left.target
                .cmp(&right.target)
                .then_with(|| left.table_commit_id.cmp(&right.table_commit_id))
        });
        Ok(receipts)
    }
}
