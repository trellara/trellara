use trellara_checkpoint::{FlowKey, PartitionWatermarkSummary, PostgresCheckpointStore};

use crate::{CliError, DatasetMode, Result, TrellaraConfig};

pub(crate) async fn target_checkpoint_store(
    config: &TrellaraConfig,
) -> Result<PostgresCheckpointStore> {
    let target = config
        .target
        .as_ref()
        .ok_or_else(|| CliError::InvalidConfig("target.database_url is required".to_string()))?;
    Ok(PostgresCheckpointStore::connect(&target.database_url, true).await?)
}

pub(crate) async fn partition_watermarks(
    config: &TrellaraConfig,
) -> Result<PartitionWatermarkSummary> {
    let DatasetMode::PartitionedScaleMode = config.dataset.mode else {
        return Err(CliError::InvalidConfig(
            "partition-watermarks requires dataset.mode=partitioned_scale_mode".to_string(),
        ));
    };
    let partition = config
        .dataset
        .partition
        .as_ref()
        .expect("validated partition settings");
    let store = target_checkpoint_store(config).await?;
    let flow = FlowKey::new(&config.source.id, &config.dataset.id);
    let checkpoints = store.load_partition_checkpoints(&flow).await?;

    Ok(PartitionWatermarkSummary::from_checkpoints(
        &config.source.id,
        &config.dataset.id,
        partition.partition_count,
        checkpoints,
    )?)
}
