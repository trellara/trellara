use std::collections::VecDeque;

use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
use tracing::error;
use trellara_protocol::TransactionEnvelope;

use crate::{
    inspect_slot_status, parse_test_decoding_event, CaptureError, ChangeSource, PgCaptureConfig,
    ReplicationSlotStatus, Result, TransactionAssembler, TransactionAssemblerConfig,
};

pub struct TestDecodingCapture {
    config: PgCaptureConfig,
    client: Client,
    assembler: TransactionAssembler,
    pending: VecDeque<TransactionEnvelope>,
}

impl TestDecodingCapture {
    pub async fn connect(config: PgCaptureConfig) -> Result<Self> {
        config.validate()?;
        let (client, connection) = tokio_postgres::connect(&config.connection_uri, NoTls).await?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                error!(%error, "test_decoding postgres connection task failed");
            }
        });

        let capture = Self {
            config,
            client,
            assembler: TransactionAssembler::default(),
            pending: VecDeque::new(),
        };
        capture.ensure_test_decoding_slot().await?;
        Ok(capture)
    }

    pub async fn ensure_test_decoding_slot(&self) -> Result<()> {
        let plugin = self
            .client
            .query_opt(
                "select plugin from pg_replication_slots where slot_name = $1",
                &[&self.config.slot_name],
            )
            .await?
            .map(|row| row.get::<_, String>("plugin"));

        match plugin {
            Some(plugin) if plugin == "test_decoding" => Ok(()),
            Some(plugin) => Err(CaptureError::SlotPluginMismatch {
                slot_name: self.config.slot_name.clone(),
                expected_plugin: "test_decoding".to_string(),
                actual_plugin: plugin,
            }),
            None if self.config.create_if_missing => {
                self.client
                    .query_one(
                        "select * from pg_create_logical_replication_slot($1, 'test_decoding')",
                        &[&self.config.slot_name],
                    )
                    .await?;
                Ok(())
            }
            None => Err(CaptureError::InvalidConfig(format!(
                "logical replication slot {} does not exist",
                self.config.slot_name
            ))),
        }
    }

    pub async fn inspect_slot_status(&self) -> Result<ReplicationSlotStatus> {
        inspect_slot_status(&self.client, &self.config.slot_name, "test_decoding").await
    }

    async fn poll_changes(&mut self) -> Result<()> {
        let rows = self
            .client
            .query(
                "select lsn::text as lsn, data from pg_logical_slot_get_changes($1, null, 100)",
                &[&self.config.slot_name],
            )
            .await?;
        let assembler_config = TransactionAssemblerConfig {
            source_id: self.config.source_id.clone(),
            database_id: self.config.database_id.clone(),
            dataset_id: self.config.dataset_id.clone(),
        };

        for row in rows {
            let lsn = row.get::<_, String>("lsn");
            let data = row.get::<_, String>("data");
            let event = parse_test_decoding_event(&lsn, &data)?;
            if let Some(envelope) = self.assembler.apply(&assembler_config, event)? {
                self.pending.push_back(envelope);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ChangeSource for TestDecodingCapture {
    async fn next_transaction(&mut self) -> Result<Option<TransactionEnvelope>> {
        if let Some(envelope) = self.pending.pop_front() {
            return Ok(Some(envelope));
        }

        self.poll_changes().await?;
        Ok(self.pending.pop_front())
    }
}
