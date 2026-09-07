#[path = "partition_rebalance/candidates.rs"]
mod candidates;
#[path = "partition_rebalance/types.rs"]
mod types;
#[path = "partition_rebalance/validation.rs"]
mod validation;

use candidates::{exceeds_skew_threshold, rebalance_candidates};
use validation::validate_rebalance_input;

use crate::ProtocolError;

pub use types::*;

pub fn plan_partition_rebalance(
    input: PartitionRebalancePlanInput,
) -> Result<PartitionRebalancePlan, ProtocolError> {
    validate_rebalance_input(&input)?;
    let missing_partitions = missing_partitions(&input);
    let blocking_partition_ids = input
        .observations
        .iter()
        .filter(|observation| observation.blocks_global_applied_watermark)
        .map(|observation| observation.partition_id)
        .collect::<Vec<_>>();
    let evidence_complete = missing_partitions.is_empty() && blocking_partition_ids.is_empty();
    let min_event_count = input
        .observations
        .iter()
        .map(|observation| observation.event_count)
        .min();
    let max_event_count = input
        .observations
        .iter()
        .map(|observation| observation.event_count)
        .max();
    let total_event_count = input
        .observations
        .iter()
        .map(|observation| observation.event_count)
        .sum();
    let skew_ratio_basis_points = min_event_count
        .zip(max_event_count)
        .and_then(|(min, max)| (min > 0).then(|| max.saturating_mul(10_000) / min));
    let skewed = evidence_complete && skewed_partition_load(&input);
    let recommended_moves = rebalance_candidates(&input, skewed);
    let status = if !evidence_complete {
        PartitionRebalanceStatus::IncompleteEvidence
    } else if skewed {
        PartitionRebalanceStatus::Skewed
    } else {
        PartitionRebalanceStatus::Stable
    };

    Ok(PartitionRebalancePlan {
        source_id: input.source_id,
        dataset_id: input.dataset_id,
        expected_partition_count: input.expected_partition_count,
        policy: input.policy,
        status,
        evidence_complete,
        runtime_movement_allowed: false,
        visibility_contract: PARTITION_REBALANCE_VISIBILITY_CONTRACT.to_string(),
        max_skew_percent: input.max_skew_percent,
        min_event_count,
        max_event_count,
        total_event_count,
        skew_ratio_basis_points,
        missing_partitions,
        blocking_partition_ids,
        recommended_moves,
    })
}

fn missing_partitions(input: &PartitionRebalancePlanInput) -> Vec<u32> {
    let observed = input
        .observations
        .iter()
        .map(|observation| observation.partition_id)
        .collect::<std::collections::BTreeSet<_>>();
    (0..input.expected_partition_count)
        .filter(|partition_id| !observed.contains(partition_id))
        .collect()
}

fn skewed_partition_load(input: &PartitionRebalancePlanInput) -> bool {
    let min = input
        .observations
        .iter()
        .map(|observation| observation.event_count)
        .min();
    let max = input
        .observations
        .iter()
        .map(|observation| observation.event_count)
        .max();
    min.zip(max)
        .is_some_and(|(min, max)| exceeds_skew_threshold(min, max, input.max_skew_percent))
}
