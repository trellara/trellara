use trellara_stream::TopicLayout;

use crate::{DatasetMode, PartitionLocalSummary, Result, TrellaraConfig};

impl PartitionLocalSummary {
    pub(crate) fn from_config(config: &TrellaraConfig) -> Result<Self> {
        let layout = TopicLayout::new(&config.source.id, &config.dataset.id)?;
        let (
            key_column,
            partition_count,
            null_key_policy,
            key_change_policy,
            manifest_topic,
            partition_topic_pattern,
        ) = match config.dataset.mode {
            DatasetMode::StrictTransactionOrder => (None, None, None, None, None, None),
            DatasetMode::PartitionedScaleMode => {
                let partition = config
                    .dataset
                    .partition
                    .as_ref()
                    .expect("validated partition settings");
                (
                    Some(partition.key_column.clone()),
                    Some(partition.partition_count),
                    Some(partition.null_key_policy.to_string()),
                    Some(partition.key_change_policy.to_string()),
                    Some(layout.manifest_topic()),
                    Some(format!(
                        "trellara.{}.{}.partition.<0..{}>",
                        config.source.id,
                        config.dataset.id,
                        partition.partition_count.saturating_sub(1)
                    )),
                )
            }
        };

        Ok(Self {
            enabled: config.dataset.mode == DatasetMode::PartitionedScaleMode,
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            key_column,
            partition_count,
            null_key_policy,
            key_change_policy,
            manifest_topic,
            partition_topic_pattern,
            guarantees: vec![
                "partition-local consumers see only their own lane".to_string(),
                "each message carries source transaction id, commit LSN, and local order metadata"
                    .to_string(),
                "single-partition transactions are marked complete for local visibility".to_string(),
            ],
            limitations: vec![
                "multi-partition transactions are not globally complete from one partition topic"
                    .to_string(),
                "use the manifest topic and barrier-aware applier when atomic full-transaction visibility is required"
                    .to_string(),
            ],
        })
    }
}
