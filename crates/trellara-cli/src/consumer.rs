use std::path::Path;

use serde::Serialize;
use trellara_stream::TopicLayout;

use crate::{
    consumer_semantics_matrix, consumer_semantics_next_steps, DatasetMode, Result, TrellaraConfig,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PartitionLocalSummary {
    pub(crate) enabled: bool,
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) key_column: Option<String>,
    pub(crate) partition_count: Option<u32>,
    pub(crate) null_key_policy: Option<String>,
    pub(crate) key_change_policy: Option<String>,
    pub(crate) manifest_topic: Option<String>,
    pub(crate) partition_topic_pattern: Option<String>,
    pub(crate) guarantees: Vec<String>,
    pub(crate) limitations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ConsumerSemanticsSummary {
    pub(crate) config: String,
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) dataset_mode: String,
    pub(crate) strict_language: String,
    pub(crate) partitioned_language: String,
    pub(crate) selected_consumer_mode: String,
    pub(crate) partition_key: Option<String>,
    pub(crate) partition_count: Option<u32>,
    pub(crate) null_key_policy: Option<String>,
    pub(crate) key_change_policy: Option<String>,
    pub(crate) manifest_topic: Option<String>,
    pub(crate) commit_topic: Option<String>,
    pub(crate) partition_topic_pattern: Option<String>,
    pub(crate) matrix: Vec<ConsumerSemanticsMode>,
    pub(crate) recommended_next_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ConsumerSemanticsMode {
    pub(crate) mode: String,
    pub(crate) best_for: String,
    pub(crate) availability: ConsumerSemanticsAvailability,
    pub(crate) transaction_boundary: String,
    pub(crate) ordering: String,
    pub(crate) latency: String,
    pub(crate) throughput: String,
    pub(crate) visibility_rule: String,
    pub(crate) consumer_obligations: Vec<String>,
    pub(crate) not_guaranteed: Vec<String>,
    pub(crate) proof_commands: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConsumerSemanticsAvailability {
    Native,
    SupportedWithBarrier,
    Advisory,
    NotRecommended,
}

impl ConsumerSemanticsSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, config_path: &Path) -> Result<Self> {
        let layout = TopicLayout::new(&config.source.id, &config.dataset.id)?;
        let config_display = config_path.display().to_string();
        let (
            selected_consumer_mode,
            partition_key,
            partition_count,
            null_key_policy,
            key_change_policy,
            manifest_topic,
            commit_topic,
            partition_topic_pattern,
        ) = match config.dataset.mode {
            DatasetMode::StrictTransactionOrder => (
                "exact_transaction".to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            DatasetMode::PartitionedScaleMode => {
                let partition = config
                    .dataset
                    .partition
                    .as_ref()
                    .expect("validated partition settings");
                (
                    "barrier_aware".to_string(),
                    Some(partition.key_column.clone()),
                    Some(partition.partition_count),
                    Some(partition.null_key_policy.to_string()),
                    Some(partition.key_change_policy.to_string()),
                    Some(layout.manifest_topic()),
                    Some(layout.commit_topic()),
                    Some(format!(
                        "trellara.{}.{}.partition.<0..{}>",
                        config.source.id,
                        config.dataset.id,
                        partition.partition_count.saturating_sub(1)
                    )),
                )
            }
        };

        let matrix = consumer_semantics_matrix(config, &config_display);
        Ok(Self {
            config: config_display.clone(),
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            dataset_mode: config.dataset.mode.to_string(),
            strict_language: "strict mode preserves source transaction order by default"
                .to_string(),
            partitioned_language:
                "partitioned mode preserves transaction identity, completeness, and per-key ordering while consumers choose between barrier-aware atomic processing and lower-latency partition-local processing"
                    .to_string(),
            selected_consumer_mode,
            partition_key,
            partition_count,
            null_key_policy,
            key_change_policy,
            manifest_topic,
            commit_topic,
            partition_topic_pattern,
            matrix,
            recommended_next_steps: consumer_semantics_next_steps(config, &config_display),
        })
    }
}
