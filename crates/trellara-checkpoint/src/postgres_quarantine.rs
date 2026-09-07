use crate::evidence::ApplyQuarantine;
use crate::postgres::PostgresCheckpointStore;
use crate::postgres_quarantine_sql::{
    CLEAR_APPLIED_TRANSACTION, CLEAR_QUARANTINE, LIST_QUARANTINE, LOAD_LATEST_QUARANTINE,
    LOAD_QUARANTINE,
};
use crate::quarantine_row::quarantine_from_row;
use crate::quarantine_validation::{
    validate_quarantine_flow, validate_quarantine_limit, validate_quarantine_transaction,
};
use crate::types::{FlowKey, TransactionKey};
use crate::Result;

impl PostgresCheckpointStore {
    pub async fn load_latest_quarantine(&self, flow: &FlowKey) -> Result<Option<ApplyQuarantine>> {
        validate_quarantine_flow(flow)?;
        self.client
            .query_opt(LOAD_LATEST_QUARANTINE, &[&flow.source_id, &flow.dataset_id])
            .await?
            .map(quarantine_from_row)
            .transpose()
    }

    pub async fn load_quarantine(
        &self,
        transaction: &TransactionKey,
    ) -> Result<Option<ApplyQuarantine>> {
        validate_quarantine_transaction(transaction)?;
        self.client
            .query_opt(
                LOAD_QUARANTINE,
                &[
                    &transaction.source_id,
                    &transaction.database_id,
                    &transaction.dataset_id,
                    &transaction.transaction_id,
                    &transaction.commit_lsn,
                ],
            )
            .await?
            .map(quarantine_from_row)
            .transpose()
    }

    pub async fn list_quarantine(
        &self,
        flow: &FlowKey,
        limit: i64,
    ) -> Result<Vec<ApplyQuarantine>> {
        validate_quarantine_flow(flow)?;
        validate_quarantine_limit(limit)?;
        self.client
            .query(
                LIST_QUARANTINE,
                &[&flow.source_id, &flow.dataset_id, &limit],
            )
            .await?
            .into_iter()
            .map(quarantine_from_row)
            .collect()
    }

    pub async fn clear_quarantine(&self, transaction: &TransactionKey) -> Result<bool> {
        validate_quarantine_transaction(transaction)?;
        Ok(self
            .client
            .execute(
                CLEAR_QUARANTINE,
                &[
                    &transaction.source_id,
                    &transaction.database_id,
                    &transaction.dataset_id,
                    &transaction.transaction_id,
                    &transaction.commit_lsn,
                ],
            )
            .await?
            > 0)
    }

    pub async fn clear_applied_transaction(&self, transaction: &TransactionKey) -> Result<bool> {
        validate_quarantine_transaction(transaction)?;
        Ok(self
            .client
            .execute(
                CLEAR_APPLIED_TRANSACTION,
                &[
                    &transaction.source_id,
                    &transaction.database_id,
                    &transaction.dataset_id,
                    &transaction.transaction_id,
                    &transaction.commit_lsn,
                ],
            )
            .await?
            > 0)
    }
}
