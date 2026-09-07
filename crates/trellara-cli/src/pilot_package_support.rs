use trellara_checkpoint::CheckpointLag;
use trellara_pg_capture::ReplicationSlotStatus;

use crate::lake_plan_tables::lake_materialization_config;
use crate::lake_writer_config::raw_cdc_writer_config;
use crate::{
    pilot_package_lake_writer_sample_envelope, resolve_lake_epoch_id, FlowStatusParts,
    FlowStatusSummary, Result, TrellaraConfig, DETERMINISTIC_EPOCH_ID,
};

pub(crate) fn pilot_package_lake_writer_plan(
    config: &TrellaraConfig,
) -> Result<trellara_lake::LakeRawCdcEpochWritePlan> {
    let envelope = pilot_package_lake_writer_sample_envelope(config)?;
    let epoch_id = resolve_lake_epoch_id(
        DETERMINISTIC_EPOCH_ID,
        config,
        std::slice::from_ref(&envelope),
    )?;
    trellara_lake::plan_raw_cdc_epoch_writes(
        &raw_cdc_writer_config(config, epoch_id, 1),
        &lake_materialization_config(config),
        &[envelope.clone(), envelope],
    )
    .map_err(Into::into)
}

pub(crate) fn pilot_package_diagnostics_status(config: &TrellaraConfig) -> FlowStatusSummary {
    let source_id = config.source.id.clone();
    let dataset_id = config.dataset.id.clone();
    let watermark_lsn = "0/16B6C50".to_string();
    let lag = CheckpointLag {
        source_id: source_id.clone(),
        dataset_id: dataset_id.clone(),
        last_seen_lsn: watermark_lsn.clone(),
        last_durable_lsn: watermark_lsn.clone(),
        last_applied_lsn: watermark_lsn,
        seen_to_durable_bytes: 0,
        durable_to_applied_bytes: 0,
        seen_to_applied_bytes: 0,
        source_is_durable: true,
        target_is_caught_up: true,
    };
    FlowStatusSummary::new(FlowStatusParts {
        source_id,
        dataset_id,
        mode: config.status_mode(),
        source_slot: ReplicationSlotStatus {
            slot_name: config.source.slot.clone(),
            exists: true,
            plugin: Some(config.source.capture.expected_plugin().to_string()),
            expected_plugin: config.source.capture.expected_plugin().to_string(),
            active: Some(false),
            failover: None,
            synced: None,
            inactive_since: None,
            idle_replication_slot_timeout: None,
            restart_lsn: Some("0/16B6B00".to_string()),
            confirmed_flush_lsn: Some("0/16B6C50".to_string()),
            retained_wal_bytes: Some(0),
            wal_status: Some("reserved".to_string()),
            safe_wal_size_bytes: Some(
                config
                    .source
                    .wal_retention_warn_bytes
                    .unwrap_or(1_000_000)
                    .max(0),
            ),
            invalidation_reason: None,
            wal_headroom: None,
            transaction_id_wraparound: None,
            xmin_horizon: None,
            issues: Vec::new(),
        },
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: config.source.wal_retention_warn_bytes,
        source_stream_spill_threshold_changes: config
            .source
            .stream_spill_threshold_changes
            .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES),
        source_stream_spill_dir: config
            .source
            .stream_spill_dir
            .as_ref()
            .map(|path| path.display().to_string()),
        source: Some(lag.clone()),
        target: config.target.as_ref().map(|_| lag),
        partition_watermarks: None,
        source_schema_drift: None,
        latest_quarantine: None,
        latest_reseed: None,
        latest_snapshot_handoff: None,
        latest_snapshot_run: None,
        latest_validation: None,
        recovery_actions: Vec::new(),
    })
}
