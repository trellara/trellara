use trellara_checkpoint::{
    ApplyQuarantine, CheckpointLag, FlowKey, PartitionWatermarkSummary, PostgresCheckpointStore,
    ReseedEvent, SnapshotHandoffEvent, SnapshotRun, ValidationEvent,
};

use crate::{DatasetMode, Result, TargetConfig, TrellaraConfig};

pub(crate) struct TargetStatusEvidence {
    pub(crate) target_lag: Option<CheckpointLag>,
    pub(crate) partition_watermarks: Option<PartitionWatermarkSummary>,
    pub(crate) latest_quarantine: Option<ApplyQuarantine>,
    pub(crate) latest_reseed: Option<ReseedEvent>,
    pub(crate) latest_snapshot_handoff: Option<SnapshotHandoffEvent>,
    pub(crate) latest_snapshot_run: Option<SnapshotRun>,
    pub(crate) latest_validation: Option<ValidationEvent>,
}

impl TargetStatusEvidence {
    pub(crate) fn empty() -> Self {
        Self {
            target_lag: None,
            partition_watermarks: None,
            latest_quarantine: None,
            latest_reseed: None,
            latest_snapshot_handoff: None,
            latest_snapshot_run: None,
            latest_validation: None,
        }
    }
}

pub(crate) async fn load_target_status_evidence(
    config: &TrellaraConfig,
    target: &TargetConfig,
) -> Result<TargetStatusEvidence> {
    let target_store = PostgresCheckpointStore::connect(&target.database_url, true).await?;
    let flow_key = FlowKey::new(&config.source.id, &config.dataset.id);
    let target_lag = trellara_relay::load_source_checkpoint(
        &target_store,
        &config.source.id,
        &config.dataset.id,
    )
    .await?
    .as_ref()
    .map(CheckpointLag::from_checkpoint);
    let partition_watermarks = load_partition_watermarks(config, &target_store, &flow_key).await?;

    Ok(TargetStatusEvidence {
        target_lag,
        partition_watermarks,
        latest_quarantine: target_store.load_latest_quarantine(&flow_key).await?,
        latest_reseed: target_store.load_latest_reseed_event(&flow_key).await?,
        latest_snapshot_handoff: target_store
            .load_latest_snapshot_handoff_event(&flow_key)
            .await?,
        latest_snapshot_run: target_store.load_latest_snapshot_run(&flow_key).await?,
        latest_validation: target_store.load_latest_validation_event(&flow_key).await?,
    })
}

async fn load_partition_watermarks(
    config: &TrellaraConfig,
    target_store: &PostgresCheckpointStore,
    flow_key: &FlowKey,
) -> Result<Option<PartitionWatermarkSummary>> {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => Ok(None),
        DatasetMode::PartitionedScaleMode => {
            let partition = config
                .dataset
                .partition
                .as_ref()
                .expect("validated partition settings");
            Ok(Some(PartitionWatermarkSummary::from_checkpoints(
                &config.source.id,
                &config.dataset.id,
                partition.partition_count,
                target_store.load_partition_checkpoints(flow_key).await?,
            )?))
        }
    }
}
