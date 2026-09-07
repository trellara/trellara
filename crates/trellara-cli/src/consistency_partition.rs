use trellara_stream::TopicLayout;

use crate::{ConsistencyPartitionContract, DatasetMode, TrellaraConfig};

pub(crate) fn consistency_partition_contract(
    config: &TrellaraConfig,
    layout: &TopicLayout,
) -> Option<ConsistencyPartitionContract> {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => None,
        DatasetMode::PartitionedScaleMode => {
            let partition = config
                .dataset
                .partition
                .as_ref()
                .expect("validated partition settings");
            Some(ConsistencyPartitionContract {
                key_column: partition.key_column.clone(),
                partition_count: partition.partition_count,
                null_key_policy: partition.null_key_policy.to_string(),
                key_change_policy: partition.key_change_policy.to_string(),
                manifest_topic: layout.manifest_topic(),
                commit_topic: layout.commit_topic(),
                partition_topic_pattern: format!(
                    "trellara.{}.{}.partition.<0..{}>",
                    config.source.id,
                    config.dataset.id,
                    partition.partition_count.saturating_sub(1)
                ),
                global_visibility_rule:
                    "barrier-aware consumers expose a source transaction only after the manifest, commit marker, and every participating partition watermark are complete"
                        .to_string(),
                partition_local_visibility_rule:
                    "partition-local consumers may process per-key ordered events before global atomic visibility and must not claim atomic visibility for cross-partition transactions"
                        .to_string(),
            })
        }
    }
}
