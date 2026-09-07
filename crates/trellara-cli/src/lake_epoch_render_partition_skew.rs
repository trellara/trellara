use std::fmt::Write as _;

use crate::LakeEpochSummary;

pub(crate) fn push_partition_skew(output: &mut String, summary: &LakeEpochSummary) {
    writeln!(
        output,
        "partition_skew: participating={} total_events={} min_events={} max_events={} ratio_bp={} hottest={} coolest={}",
        summary.partition_skew.participating_partition_count,
        summary.partition_skew.total_event_count,
        summary.partition_skew.min_event_count,
        summary.partition_skew.max_event_count,
        summary
            .partition_skew
            .skew_ratio_basis_points
            .map(|ratio| ratio.to_string())
            .unwrap_or_else(|| "none".to_string()),
        partition_ids(&summary.partition_skew.hottest_partition_ids),
        partition_ids(&summary.partition_skew.coolest_partition_ids)
    )
    .expect("write string");
}

fn partition_ids(partition_ids: &[u32]) -> String {
    if partition_ids.is_empty() {
        return "none".to_string();
    }
    partition_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
