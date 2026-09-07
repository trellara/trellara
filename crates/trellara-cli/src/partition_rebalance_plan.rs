use std::collections::BTreeMap;

use trellara_protocol::{
    plan_partition_rebalance, PartitionLoadObservation, PartitionRebalancePlan,
    PartitionRebalancePlanInput, PartitionRebalancePolicy,
};

use crate::{CliError, PartitionRebalancePlanArgs, Result};
use trellara_checkpoint::PartitionWatermarkSummary;

pub(crate) fn partition_rebalance_plan_from_watermarks(
    watermarks: &PartitionWatermarkSummary,
    args: &PartitionRebalancePlanArgs,
) -> Result<PartitionRebalancePlan> {
    let event_counts = parse_partition_event_counts(&args.partition_event_counts)?;
    let observations = watermarks
        .partitions
        .iter()
        .map(|partition| {
            let event_count = event_counts
                .get(&partition.partition_id)
                .copied()
                .ok_or_else(|| {
                    CliError::InvalidConfig(format!(
                        "missing --partition-event-count {}=<count>",
                        partition.partition_id
                    ))
                })?;
            Ok(PartitionLoadObservation {
                partition_id: partition.partition_id,
                event_count,
                durable_lsn: partition.last_durable_lsn.clone(),
                applied_lsn: partition.last_applied_lsn.clone(),
                blocks_global_applied_watermark: partition.blocks_global_applied_watermark,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(plan_partition_rebalance(PartitionRebalancePlanInput {
        source_id: watermarks.source_id.clone(),
        dataset_id: watermarks.dataset_id.clone(),
        expected_partition_count: watermarks.expected_partition_count,
        policy: rebalance_policy(args.plan_moves),
        max_skew_percent: args.max_skew_percent,
        observations,
    })?)
}

fn parse_partition_event_counts(values: &[String]) -> Result<BTreeMap<u32, u64>> {
    let mut counts = BTreeMap::new();
    for value in values {
        let (partition, count) = value.split_once('=').ok_or_else(|| {
            CliError::InvalidConfig(format!(
                "partition event count {value:?} must use PARTITION=COUNT"
            ))
        })?;
        let partition_id = partition.parse::<u32>().map_err(|_| {
            CliError::InvalidConfig(format!(
                "partition id {partition:?} must be an unsigned integer"
            ))
        })?;
        let event_count = count.parse::<u64>().map_err(|_| {
            CliError::InvalidConfig(format!("event count {count:?} must be an unsigned integer"))
        })?;
        if counts.insert(partition_id, event_count).is_some() {
            return Err(CliError::InvalidConfig(format!(
                "duplicate --partition-event-count for partition {partition_id}"
            )));
        }
    }
    Ok(counts)
}

fn rebalance_policy(plan_moves: bool) -> PartitionRebalancePolicy {
    if plan_moves {
        PartitionRebalancePolicy::PlanIfSkewed
    } else {
        PartitionRebalancePolicy::ObserveOnly
    }
}
