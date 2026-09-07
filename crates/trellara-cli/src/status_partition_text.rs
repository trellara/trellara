use std::fmt::Write as _;

use trellara_checkpoint::{PartitionScaleHealthSummary, PartitionWatermarkSummary};

use format::{csv_str, csv_u32, option_u64, partition_scale_health_status_label};

#[path = "status_partition_text/format.rs"]
mod format;

pub(crate) fn render_partition_watermark_text(summary: &PartitionWatermarkSummary) -> String {
    let health = PartitionScaleHealthSummary::from_watermark_summary(summary);
    let mut output = String::new();
    writeln!(&mut output, "Trellara partition watermarks").expect("write to string");
    push_summary_lines(&mut output, summary, &health);
    push_partition_lines(&mut output, summary);
    output
}

fn push_summary_lines(
    output: &mut String,
    summary: &PartitionWatermarkSummary,
    health: &PartitionScaleHealthSummary,
) {
    writeln!(output, "source_id: {}", summary.source_id).expect("write to string");
    writeln!(output, "dataset_id: {}", summary.dataset_id).expect("write to string");
    writeln!(
        output,
        "expected_partition_count: {}",
        summary.expected_partition_count
    )
    .expect("write to string");
    writeln!(
        output,
        "observed_partition_count: {}",
        summary.observed_partition_count
    )
    .expect("write to string");
    writeln!(
        output,
        "complete_partition_set: {}",
        summary.complete_partition_set
    )
    .expect("write to string");
    writeln!(
        output,
        "global_durable_lsn: {}",
        summary.global_durable_lsn.as_deref().unwrap_or("unknown")
    )
    .expect("write to string");
    writeln!(
        output,
        "global_applied_lsn: {}",
        summary.global_applied_lsn.as_deref().unwrap_or("unknown")
    )
    .expect("write to string");
    writeln!(
        output,
        "global_durable_to_applied_bytes: {}",
        option_u64(&summary.global_durable_to_applied_bytes)
    )
    .expect("write to string");
    push_health_lines(output, summary, health);
}

fn push_health_lines(
    output: &mut String,
    summary: &PartitionWatermarkSummary,
    health: &PartitionScaleHealthSummary,
) {
    writeln!(
        output,
        "missing_partitions: {}",
        csv_u32(&summary.missing_partitions)
    )
    .expect("write to string");
    writeln!(
        output,
        "partition_scale_health: {}",
        partition_scale_health_status_label(&health.status)
    )
    .expect("write to string");
    writeln!(
        output,
        "blocking_partitions: {}",
        csv_u32(&health.blocking_partition_ids)
    )
    .expect("write to string");
    writeln!(
        output,
        "global_visibility_releasable: {}",
        health.global_visibility_releasable
    )
    .expect("write to string");
    writeln!(
        output,
        "global_visibility_blocker_codes: {}",
        csv_str(&health.global_visibility_blocker_codes)
    )
    .expect("write to string");
    writeln!(
        output,
        "global_durable_low_watermark_partitions: {}",
        csv_u32(&health.global_durable_low_watermark_partition_ids)
    )
    .expect("write to string");
    writeln!(
        output,
        "global_applied_low_watermark_partitions: {}",
        csv_u32(&health.global_applied_low_watermark_partition_ids)
    )
    .expect("write to string");
    writeln!(
        output,
        "lagging_partitions: {}",
        csv_u32(&health.lagging_partition_ids)
    )
    .expect("write to string");
    writeln!(
        output,
        "straggler_partitions: {}",
        csv_u32(&health.straggler_partition_ids)
    )
    .expect("write to string");
    writeln!(
        output,
        "observed_applied_skew_bytes: {}",
        option_u64(&health.observed_applied_skew_bytes)
    )
    .expect("write to string");
    writeln!(
        output,
        "max_partition_lag_bytes: {}",
        option_u64(&health.max_partition_lag_bytes)
    )
    .expect("write to string");
    writeln!(
        output,
        "visibility_actions: {}",
        health.visibility_actions.len()
    )
    .expect("write to string");
    for action in &health.visibility_actions {
        writeln!(
            output,
            "- action: {} partitions: {} command: {} reason: {}",
            action.code,
            csv_u32(&action.partition_ids),
            action.command,
            action.reason
        )
        .expect("write to string");
    }
}

fn push_partition_lines(output: &mut String, summary: &PartitionWatermarkSummary) {
    writeln!(output, "partitions: {}", summary.partitions.len()).expect("write to string");

    for partition in &summary.partitions {
        writeln!(
            output,
            "- partition_id: {} durable_lsn: {} applied_lsn: {} durable_to_applied_bytes: {} blocks_global_applied_watermark: {}",
            partition.partition_id,
            partition.last_durable_lsn,
            partition.last_applied_lsn,
            partition.durable_to_applied_bytes,
            partition.blocks_global_applied_watermark
        )
        .expect("write to string");
    }
}
