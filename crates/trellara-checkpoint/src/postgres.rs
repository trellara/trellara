use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
use trellara_protocol::Checkpoint;

use crate::checkpoint_rewind::reject_checkpoint_rewind;
use crate::checkpoint_row::checkpoint_from_row;
use crate::checkpoint_validation::{validate_checkpoint, validate_flow_key};
use crate::postgres_sql::{
    APPLIED_TRANSACTION_EXISTS, INSERT_APPLIED_TRANSACTION, LOAD_CHECKPOINT, UPSERT_CHECKPOINT,
};
use crate::schema::postgres_checkpoint_schema_sql;
use crate::transaction_key_validation::validate_transaction_key;
use crate::types::{ApplyDecision, CheckpointStore, DedupStore, FlowKey, TransactionKey};
use crate::Result;

pub struct PostgresCheckpointStore {
    pub(crate) client: Client,
}

impl PostgresCheckpointStore {
    pub async fn connect(connection_uri: &str, ensure_schema: bool) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(connection_uri, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("postgres checkpoint connection failed: {error}");
            }
        });

        let store = Self { client };
        if ensure_schema {
            store.ensure_schema().await?;
        }
        Ok(store)
    }

    pub async fn ensure_schema(&self) -> Result<()> {
        self.client
            .batch_execute(postgres_checkpoint_schema_sql())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl CheckpointStore for PostgresCheckpointStore {
    async fn load_checkpoint(&self, flow: &FlowKey) -> Result<Option<Checkpoint>> {
        validate_flow_key(flow)?;
        self.client
            .query_opt(LOAD_CHECKPOINT, &[&flow.source_id, &flow.dataset_id])
            .await?
            .map(checkpoint_from_row)
            .transpose()
    }

    async fn save_checkpoint(&self, checkpoint: Checkpoint) -> Result<()> {
        validate_checkpoint(&checkpoint)?;
        let flow = FlowKey::new(&checkpoint.source_id, &checkpoint.dataset_id);
        if let Some(existing) = self.load_checkpoint(&flow).await? {
            reject_checkpoint_rewind(&existing, &checkpoint)?;
        }
        self.client
            .execute(
                UPSERT_CHECKPOINT,
                &[
                    &checkpoint.source_id,
                    &checkpoint.dataset_id,
                    &checkpoint.last_seen_lsn,
                    &checkpoint.last_durable_lsn,
                    &checkpoint.last_applied_lsn,
                ],
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl DedupStore for PostgresCheckpointStore {
    async fn apply_decision(&self, transaction: &TransactionKey) -> Result<ApplyDecision> {
        validate_transaction_key(transaction)?;
        let exists = self
            .client
            .query_opt(
                APPLIED_TRANSACTION_EXISTS,
                &[
                    &transaction.source_id,
                    &transaction.database_id,
                    &transaction.dataset_id,
                    &transaction.transaction_id,
                    &transaction.commit_lsn,
                ],
            )
            .await?
            .is_some();

        if exists {
            Ok(ApplyDecision::SkipDuplicate)
        } else {
            Ok(ApplyDecision::Apply)
        }
    }

    async fn record_applied(&self, transaction: TransactionKey) -> Result<()> {
        validate_transaction_key(&transaction)?;
        self.client
            .execute(
                INSERT_APPLIED_TRANSACTION,
                &[
                    &transaction.source_id,
                    &transaction.database_id,
                    &transaction.dataset_id,
                    &transaction.transaction_id,
                    &transaction.commit_lsn,
                ],
            )
            .await?;
        Ok(())
    }
}
