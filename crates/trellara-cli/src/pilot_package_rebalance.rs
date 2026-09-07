use trellara_protocol::{
    plan_partition_rebalance, PartitionLoadObservation, PartitionRebalancePlan,
    PartitionRebalancePlanInput, PartitionRebalancePolicy,
};

use crate::{CliError, DatasetMode, Result, TrellaraConfig};

pub(crate) fn pilot_package_partition_rebalance_plan(
    config: &TrellaraConfig,
) -> Result<Option<PartitionRebalancePlan>> {
    if config.dataset.mode != DatasetMode::PartitionedScaleMode {
        return Ok(None);
    }

    let partition = config.dataset.partition.as_ref().ok_or_else(|| {
        CliError::InvalidConfig(
            "dataset.partition is required for partitioned package rebalance evidence".to_string(),
        )
    })?;

    Ok(Some(plan_partition_rebalance(
        PartitionRebalancePlanInput {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            expected_partition_count: partition.partition_count,
            policy: PartitionRebalancePolicy::PlanIfSkewed,
            max_skew_percent: 50,
            observations: sample_partition_load(partition.partition_count),
        },
    )?))
}

fn sample_partition_load(partition_count: u32) -> Vec<PartitionLoadObservation> {
    (0..partition_count)
        .map(|partition_id| PartitionLoadObservation {
            partition_id,
            event_count: sample_event_count(partition_id),
            durable_lsn: "0/16B9000".to_string(),
            applied_lsn: "0/16B9000".to_string(),
            blocks_global_applied_watermark: false,
        })
        .collect()
}

fn sample_event_count(partition_id: u32) -> u64 {
    if partition_id == 0 {
        1_000
    } else {
        100
    }
}
