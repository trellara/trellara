use std::collections::VecDeque;
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::timeout;
use trellara_protocol::TransactionEnvelope;

use crate::assembler_config::TransactionAssemblerConfig;
use crate::pgoutput_stream_start::pgoutput_replication_start_plan;
use crate::replication::ReplicationBootstrapConnection;
use crate::{
    CaptureBootstrap, ChangeSource, PgCapture, PgCaptureConfig, PgChangeSource, PgOutputDecoder,
    Result, TransactionAssembler,
};

pub struct PgOutputStreamCapture {
    config: PgCaptureConfig,
    bootstrap: CaptureBootstrap,
    connection: ReplicationBootstrapConnection,
    decoder: PgOutputDecoder,
    assembler: TransactionAssembler,
    pending: VecDeque<TransactionEnvelope>,
    idle_timeout: Duration,
}

impl PgOutputStreamCapture {
    pub async fn connect(config: PgCaptureConfig) -> Result<Self> {
        config.validate()?;
        let control = PgCapture::connect(config.clone()).await?;
        control.ensure_publication().await?;
        let slot = control.ensure_logical_slot().await?;
        control.ensure_slot_plugin("pgoutput").await?;
        let relations = control.load_relations().await?;
        let preflight = control.ensure_capture_safe().await?;
        let bootstrap = CaptureBootstrap {
            publication_name: config.publication_name.clone(),
            slot_name: config.slot_name.clone(),
            consistent_lsn: slot.consistent_lsn.clone(),
            exported_snapshot_name: None,
            relations,
            preflight,
        };

        let mut connection =
            ReplicationBootstrapConnection::connect(&config.connection_uri).await?;
        let replication_start = pgoutput_replication_start_plan(&slot, &config)?;
        let options = replication_start.options(&config);
        connection
            .start_logical_replication(&config.slot_name, &replication_start.start_lsn, &options)
            .await?;

        Ok(Self {
            assembler: TransactionAssembler::with_stream_spill_config(
                config.stream_spill_threshold_changes,
                config.stream_spill_dir.clone(),
            ),
            config,
            bootstrap,
            connection,
            decoder: PgOutputDecoder::default(),
            pending: VecDeque::new(),
            idle_timeout: Duration::from_millis(250),
        })
    }

    async fn poll_replication_message(&mut self) -> Result<()> {
        let Some(xlog_data) = self.connection.read_xlog_data().await? else {
            return Ok(());
        };
        let assembler_config = TransactionAssemblerConfig {
            source_id: self.config.source_id.clone(),
            database_id: self.config.database_id.clone(),
            dataset_id: self.config.dataset_id.clone(),
        };

        if let Some(event) = self.decoder.decode(&xlog_data.payload)? {
            if let Some(envelope) = self.assembler.apply(&assembler_config, event)? {
                self.pending.push_back(envelope);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl PgChangeSource for PgOutputStreamCapture {
    async fn bootstrap(&mut self) -> Result<CaptureBootstrap> {
        Ok(self.bootstrap.clone())
    }

    async fn next_transaction(&mut self) -> Result<Option<TransactionEnvelope>> {
        if let Some(envelope) = self.pending.pop_front() {
            return Ok(Some(envelope));
        }

        loop {
            match timeout(self.idle_timeout, self.poll_replication_message()).await {
                Ok(result) => result?,
                Err(_) => return Ok(None),
            }
            if let Some(envelope) = self.pending.pop_front() {
                return Ok(Some(envelope));
            }
        }
    }

    async fn acknowledge_durable(&mut self, lsn: &str) -> Result<()> {
        self.connection.send_standby_status_update(lsn).await
    }
}

#[async_trait]
impl ChangeSource for PgOutputStreamCapture {
    async fn next_transaction(&mut self) -> Result<Option<TransactionEnvelope>> {
        PgChangeSource::next_transaction(self).await
    }

    async fn acknowledge_durable_lsn(&mut self, lsn: &str) -> Result<()> {
        self.acknowledge_durable(lsn).await
    }
}
