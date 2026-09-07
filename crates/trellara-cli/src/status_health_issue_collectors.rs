use trellara_checkpoint::{
    ApplyQuarantine, CheckpointLag, PartitionWatermarkSummary, ValidationEvent,
};

use crate::{
    validation_drift_message, validation_stale_message, TargetSourceProgress, ValidationProgress,
};

pub(crate) fn collect_checkpoint_issues(
    issues: &mut Vec<String>,
    source: Option<&CheckpointLag>,
    target: Option<&CheckpointLag>,
) {
    match source {
        Some(source) if !source.source_is_durable => issues.push(format!(
            "source checkpoint durable LSN {} is behind seen LSN {} by {} bytes",
            source.last_durable_lsn, source.last_seen_lsn, source.seen_to_durable_bytes
        )),
        None => issues.push("source checkpoint is missing".to_string()),
        _ => {}
    }

    match target {
        Some(target) if !target.target_is_caught_up => issues.push(format!(
            "target applied LSN {} is behind durable LSN {} by {} bytes",
            target.last_applied_lsn, target.last_durable_lsn, target.durable_to_applied_bytes
        )),
        None => issues.push("target checkpoint is missing".to_string()),
        _ => {}
    }

    collect_source_to_target_issue(issues, source, target);
}

fn collect_source_to_target_issue(
    issues: &mut Vec<String>,
    source: Option<&CheckpointLag>,
    target: Option<&CheckpointLag>,
) {
    let progress = TargetSourceProgress::from_lags(source, target);
    if let (Some(source), Some(target)) = (source, target) {
        if target.target_is_caught_up && !progress.reaches_source_durable {
            issues.push(format!(
                "target applied LSN {} is behind source durable LSN {} by {} bytes",
                target.last_applied_lsn,
                source.last_durable_lsn,
                progress.source_to_target_lag_bytes.unwrap_or_default()
            ));
        }
    }
}

pub(crate) fn collect_partition_issues(
    issues: &mut Vec<String>,
    partition_watermarks: Option<&PartitionWatermarkSummary>,
) {
    if let Some(watermarks) = partition_watermarks {
        if !watermarks.complete_partition_set {
            issues.push(format!(
                "partition watermarks missing {} of {} partitions",
                watermarks.missing_partitions.len(),
                watermarks.expected_partition_count
            ));
        } else if watermarks
            .global_durable_to_applied_bytes
            .is_some_and(|lag| lag > 0)
        {
            issues.push(partition_lag_issue(watermarks));
        }
    }
}

fn partition_lag_issue(watermarks: &PartitionWatermarkSummary) -> String {
    let blocking_partitions = watermarks
        .partitions
        .iter()
        .filter(|partition| partition.blocks_global_applied_watermark)
        .map(|partition| partition.partition_id.to_string())
        .collect::<Vec<_>>();
    format!(
        "partition global applied watermark {} is behind durable watermark {} by {} bytes; blocking partitions: {}",
        watermarks.global_applied_lsn.as_deref().unwrap_or(""),
        watermarks.global_durable_lsn.as_deref().unwrap_or(""),
        watermarks.global_durable_to_applied_bytes.unwrap_or_default(),
        if blocking_partitions.is_empty() {
            "unknown".to_string()
        } else {
            blocking_partitions.join(",")
        }
    )
}

pub(crate) fn collect_failure_issues(
    issues: &mut Vec<String>,
    latest_quarantine: Option<&ApplyQuarantine>,
    latest_validation: Option<&ValidationEvent>,
    source: Option<&CheckpointLag>,
    target: Option<&CheckpointLag>,
) {
    if let Some(quarantine) = latest_quarantine {
        issues.push(format!(
            "target quarantine contains transaction {} at LSN {}: {}",
            quarantine.transaction_id, quarantine.commit_lsn, quarantine.reason
        ));
    }

    if let Some(validation) = latest_validation.filter(|validation| !validation.converged) {
        issues.push(validation_drift_message(validation));
    }
    if let Some(validation) = latest_validation.filter(|validation| validation.converged) {
        let progress = ValidationProgress::from_parts(validation, source, target);
        if !progress.is_current {
            issues.push(validation_stale_message(progress));
        }
    }
}
