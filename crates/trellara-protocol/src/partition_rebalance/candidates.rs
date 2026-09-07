use super::{
    PartitionRebalanceMoveCandidate, PartitionRebalancePlanInput, PartitionRebalancePolicy,
};

pub(super) fn rebalance_candidates(
    input: &PartitionRebalancePlanInput,
    skewed: bool,
) -> Vec<PartitionRebalanceMoveCandidate> {
    if !skewed || input.policy != PartitionRebalancePolicy::PlanIfSkewed {
        return Vec::new();
    }
    let Some(from) = input.observations.iter().max_by_key(|row| row.event_count) else {
        return Vec::new();
    };
    let Some(to) = input.observations.iter().min_by_key(|row| row.event_count) else {
        return Vec::new();
    };
    vec![PartitionRebalanceMoveCandidate {
        from_partition_id: from.partition_id,
        to_partition_id: to.partition_id,
        estimated_event_delta: from.event_count.saturating_sub(to.event_count) / 2,
        reason: format!(
            "partition {} has {} events while partition {} has {} events",
            from.partition_id, from.event_count, to.partition_id, to.event_count
        ),
    }]
}

pub(super) fn exceeds_skew_threshold(min: u64, max: u64, max_skew_percent: u32) -> bool {
    max.saturating_mul(100) > min.saturating_mul(u64::from(100 + max_skew_percent))
}
