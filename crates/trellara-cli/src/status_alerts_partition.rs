use crate::{FlowAlert, FlowStatusSummary};

pub(crate) fn partition_alerts(status: &FlowStatusSummary) -> Vec<FlowAlert> {
    let Some(watermarks) = status.partition_watermarks.as_ref() else {
        return Vec::new();
    };

    if !watermarks.complete_partition_set {
        return vec![FlowAlert::warning(
            "partition_watermark_incomplete",
            format!(
                "partition watermarks are missing {} of {} partitions",
                watermarks.missing_partitions.len(),
                watermarks.expected_partition_count
            ),
            "run the barrier-aware applier for all partition topics before exposing global partitioned visibility",
        )];
    }

    if watermarks
        .global_durable_to_applied_bytes
        .is_some_and(|lag| lag > 0)
    {
        return vec![FlowAlert::warning(
            "partition_global_watermark_lag",
            format!(
                "partition global applied watermark {} is behind durable watermark {} by {} bytes",
                watermarks.global_applied_lsn.as_deref().unwrap_or(""),
                watermarks.global_durable_lsn.as_deref().unwrap_or(""),
                watermarks.global_durable_to_applied_bytes.unwrap_or_default()
            ),
            "drain lagging partition consumers before using current-state or epoch-style visibility",
        )];
    }

    Vec::new()
}
