use crate::{
    connect_control_client, fail_on_preflight_issues, fail_on_slot_plugin_mismatch,
    inspect_logical_replication_slots, inspect_slot_status, inspect_subscription_conflict_stats,
    inspect_tables, CaptureBootstrap, ExportedCaptureBootstrap, PgCapture, PgCaptureConfig,
    ReplicationSlotStatus, Result, SubscriptionConflictStats, TablePreflight,
};

impl PgCapture {
    pub async fn connect(config: PgCaptureConfig) -> Result<Self> {
        config.validate()?;
        let client = connect_control_client(&config.connection_uri).await?;

        Ok(Self { config, client })
    }

    pub async fn bootstrap(config: PgCaptureConfig) -> Result<CaptureBootstrap> {
        let capture = Self::connect(config).await?;
        capture.ensure_publication().await?;
        let slot = capture.ensure_logical_slot().await?;
        capture.ensure_slot_plugin("pgoutput").await?;
        let relations = capture.load_relations().await?;
        let preflight = capture.inspect_tables().await?;
        fail_on_preflight_issues(&preflight)?;

        Ok(CaptureBootstrap {
            publication_name: capture.config.publication_name,
            slot_name: capture.config.slot_name,
            consistent_lsn: slot.consistent_lsn,
            exported_snapshot_name: None,
            relations,
            preflight,
        })
    }

    pub async fn bootstrap_with_exported_snapshot(
        config: PgCaptureConfig,
    ) -> Result<ExportedCaptureBootstrap> {
        let capture = Self::connect(config.clone()).await?;
        capture.ensure_publication().await?;
        let exported_slot = Self::create_exported_logical_slot(config).await?;
        let relations = capture.load_relations().await?;
        let preflight = capture.inspect_tables().await?;
        fail_on_preflight_issues(&preflight)?;

        Ok(ExportedCaptureBootstrap {
            publication_name: capture.config.publication_name,
            relations,
            preflight,
            exported_slot,
        })
    }

    pub async fn inspect_tables(&self) -> Result<Vec<TablePreflight>> {
        inspect_tables(&self.client, &self.config.tables).await
    }

    pub async fn inspect_slot_status(
        &self,
        expected_plugin: &str,
    ) -> Result<ReplicationSlotStatus> {
        inspect_slot_status(&self.client, &self.config.slot_name, expected_plugin).await
    }

    pub async fn inspect_logical_replication_slots(&self) -> Result<Vec<ReplicationSlotStatus>> {
        inspect_logical_replication_slots(&self.client).await
    }

    pub async fn ensure_slot_plugin(&self, expected_plugin: &str) -> Result<()> {
        let status = self.inspect_slot_status(expected_plugin).await?;
        fail_on_slot_plugin_mismatch(&status)
    }

    pub async fn inspect_subscription_conflict_stats(
        &self,
    ) -> Result<Vec<SubscriptionConflictStats>> {
        inspect_subscription_conflict_stats(&self.client).await
    }

    pub async fn ensure_capture_safe(&self) -> Result<Vec<TablePreflight>> {
        let preflight = self.inspect_tables().await?;
        fail_on_preflight_issues(&preflight)?;
        Ok(preflight)
    }
}
