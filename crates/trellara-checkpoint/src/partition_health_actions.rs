use crate::{PartitionScaleHealthAction, PartitionScaleHealthStatus, PartitionWatermarkSummary};

pub(crate) fn partition_visibility_actions(
    summary: &PartitionWatermarkSummary,
    status: &PartitionScaleHealthStatus,
    blocking_partition_ids: &[u32],
    straggler_partition_ids: &[u32],
) -> Vec<PartitionScaleHealthAction> {
    let mut actions = Vec::new();
    if !summary.missing_partitions.is_empty() {
        actions.push(missing_partitions_action(summary));
    }
    if !blocking_partition_ids.is_empty() {
        actions.push(blocking_partitions_action(blocking_partition_ids));
    }
    if *status == PartitionScaleHealthStatus::LaggingPartitions
        && summary
            .partitions
            .iter()
            .any(|partition| partition.durable_to_applied_bytes > 0)
    {
        actions.push(lagging_partitions_action(summary, straggler_partition_ids));
    }
    actions
}

fn missing_partitions_action(summary: &PartitionWatermarkSummary) -> PartitionScaleHealthAction {
    PartitionScaleHealthAction {
        code: "collect_missing_partition_watermarks".to_string(),
        partition_ids: summary.missing_partitions.clone(),
        command: "trellara partition-watermarks --config <flow>".to_string(),
        reason: format!(
            "global visibility is withheld until partitions {} report durable and applied LSNs",
            csv_u32(&summary.missing_partitions)
        ),
    }
}

fn blocking_partitions_action(partition_ids: &[u32]) -> PartitionScaleHealthAction {
    PartitionScaleHealthAction {
        code: "advance_global_applied_low_watermark".to_string(),
        partition_ids: partition_ids.to_vec(),
        command: "trellara apply --config <flow>".to_string(),
        reason: format!(
            "partition(s) {} define the global applied low watermark and block newer partition-local progress from becoming globally visible",
            csv_u32(partition_ids)
        ),
    }
}

fn lagging_partitions_action(
    summary: &PartitionWatermarkSummary,
    straggler_partition_ids: &[u32],
) -> PartitionScaleHealthAction {
    let partition_ids = if straggler_partition_ids.is_empty() {
        summary
            .partitions
            .iter()
            .filter(|partition| partition.durable_to_applied_bytes > 0)
            .map(|partition| partition.partition_id)
            .collect()
    } else {
        straggler_partition_ids.to_vec()
    };
    PartitionScaleHealthAction {
        code: "drain_lagging_partition_lanes".to_string(),
        partition_ids,
        command: "trellara apply --config <flow>".to_string(),
        reason: "partitioned scale mode is not ready until every partition lane catches up to its durable watermark"
            .to_string(),
    }
}

fn csv_u32(values: &[u32]) -> String {
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
