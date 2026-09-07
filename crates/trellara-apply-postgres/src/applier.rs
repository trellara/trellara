use std::collections::HashMap;

use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
use tracing::error;
use trellara_checkpoint::postgres_checkpoint_schema_sql;
use trellara_protocol::TransactionEnvelope;

use crate::checkpoint_quarantine::record_quarantined_envelope;
use crate::{
    apply_envelope_transactionally, ApplyOutcome, ApplyTablePolicy, EnvelopeApplier,
    PostgresApplyConfig, Result,
};

pub struct PostgresApplier {
    pub(super) client: Client,
    ensure_checkpoint_schema: bool,
    table_policies: HashMap<String, ApplyTablePolicy>,
}

impl PostgresApplier {
    pub async fn connect(config: PostgresApplyConfig) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(&config.connection_uri, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                error!(%error, "postgres applier connection task failed");
            }
        });

        let applier = Self {
            client,
            ensure_checkpoint_schema: config.ensure_checkpoint_schema,
            table_policies: config
                .table_policies
                .into_iter()
                .map(|policy| (policy.relation.display_name(), policy))
                .collect(),
        };

        if applier.ensure_checkpoint_schema {
            applier.ensure_schema().await?;
        }

        Ok(applier)
    }

    pub async fn ensure_schema(&self) -> Result<()> {
        self.client
            .batch_execute(postgres_checkpoint_schema_sql())
            .await?;
        Ok(())
    }

    pub async fn apply_envelope(&mut self, envelope: &TransactionEnvelope) -> Result<ApplyOutcome> {
        envelope.verify_checksum()?;

        let result = self.apply_envelope_transactionally(envelope).await;
        if let Err(apply_error) = &result {
            if let Err(quarantine_error) =
                record_quarantined_envelope(&self.client, envelope, apply_error).await
            {
                error!(
                    transaction_id = %envelope.transaction_id,
                    commit_lsn = %envelope.commit_lsn,
                    %quarantine_error,
                    "failed to record quarantined apply transaction"
                );
            }
        }

        result
    }

    async fn apply_envelope_transactionally(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<ApplyOutcome> {
        apply_envelope_transactionally(&mut self.client, &self.table_policies, envelope).await
    }
}

#[async_trait]
impl EnvelopeApplier for PostgresApplier {
    async fn apply_envelope(&mut self, envelope: &TransactionEnvelope) -> Result<ApplyOutcome> {
        PostgresApplier::apply_envelope(self, envelope).await
    }
}
