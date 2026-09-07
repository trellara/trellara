use trellara_checkpoint::PartitionWatermarkSummary;

use crate::{is_partitioned_mode, RepairPlanStep};

pub(crate) fn partition_watermark_repair_steps(
    mode: &str,
    watermarks: Option<&PartitionWatermarkSummary>,
    config_path: &str,
    offset: usize,
) -> Vec<RepairPlanStep> {
    if !is_partitioned_mode(mode) {
        return Vec::new();
    }
    let Some(reason) = partition_watermark_repair_reason(watermarks) else {
        return Vec::new();
    };

    [
        (
            "partition_watermark_apply",
            format!("trellara apply --config {config_path}"),
            "drain the barrier-aware applier so every participating partition can report an applied watermark",
        ),
        (
            "partition_watermark_inspect",
            format!("trellara partition-watermarks --config {config_path}"),
            "confirm the global partition low watermark is complete before exposing global partitioned visibility",
        ),
        (
            "partition_watermark_verify",
            format!("trellara verify --config {config_path}"),
            "prove checksum convergence after partitioned visibility catches up",
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (action_code, command, hint))| RepairPlanStep {
        order: offset + index + 1,
        action_code: action_code.to_string(),
        reason: reason.clone(),
        command,
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint: hint.to_string(),
    })
    .collect()
}

fn partition_watermark_repair_reason(
    watermarks: Option<&PartitionWatermarkSummary>,
) -> Option<String> {
    match watermarks {
        None => Some("partitioned scale mode has no partition watermark evidence".to_string()),
        Some(watermarks) if !watermarks.complete_partition_set => Some(format!(
            "partition watermarks are missing {} of {} partitions",
            watermarks.missing_partitions.len(),
            watermarks.expected_partition_count
        )),
        Some(watermarks) => {
            let lag = watermarks.global_durable_to_applied_bytes?;
            if lag == 0 {
                return None;
            }
            Some(format!(
                "partition global applied watermark {} is behind durable watermark {} by {} bytes",
                watermarks.global_applied_lsn.as_deref().unwrap_or(""),
                watermarks.global_durable_lsn.as_deref().unwrap_or(""),
                lag
            ))
        }
    }
}
