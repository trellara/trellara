use crate::{is_partitioned_mode, FlowStatusSummary, TargetSourceProgress};

pub(crate) fn transaction_boundary_evidence(
    status: &FlowStatusSummary,
    manifest_barrier_required: bool,
    manifest_barrier_complete: Option<bool>,
    global_partition_watermark_caught_up: Option<bool>,
) -> String {
    let mut parts = Vec::new();

    parts.push(match status.source.as_ref() {
        Some(source) => format!(
            "source_checkpoint last_seen_lsn={} last_durable_lsn={} durable={}",
            source.last_seen_lsn, source.last_durable_lsn, source.source_is_durable
        ),
        None => "source_checkpoint missing".to_string(),
    });
    parts.push(match status.target.as_ref() {
        Some(target) => format!(
            "target_checkpoint last_durable_lsn={} last_applied_lsn={} caught_up={}",
            target.last_durable_lsn, target.last_applied_lsn, target.target_is_caught_up
        ),
        None => "target_checkpoint missing".to_string(),
    });
    parts.push(target_source_progress_evidence(status));
    parts.push(format!(
        "manifest_barrier required={} complete={}",
        manifest_barrier_required,
        optional_bool_evidence(manifest_barrier_complete)
    ));
    if is_partitioned_mode(&status.mode) {
        parts.push(match status.partition_watermarks.as_ref() {
            Some(watermarks) => format!(
                "partition_watermark observed_partitions={}/{} complete={} global_durable_lsn={} global_applied_lsn={} global_lag_bytes={} caught_up={}",
                watermarks.observed_partition_count,
                watermarks.expected_partition_count,
                watermarks.complete_partition_set,
                watermarks.global_durable_lsn.as_deref().unwrap_or("unknown"),
                watermarks.global_applied_lsn.as_deref().unwrap_or("unknown"),
                watermarks
                    .global_durable_to_applied_bytes
                    .map(|bytes| bytes.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                optional_bool_evidence(global_partition_watermark_caught_up)
            ),
            None => "partition_watermark missing".to_string(),
        });
    }
    parts.push(match status.latest_quarantine.as_ref() {
        Some(quarantine) => format!(
            "target_quarantine transaction_id={} commit_lsn={} reason={}",
            quarantine.transaction_id, quarantine.commit_lsn, quarantine.reason
        ),
        None => "target_quarantine none".to_string(),
    });

    parts.join("; ")
}

fn target_source_progress_evidence(status: &FlowStatusSummary) -> String {
    let progress = TargetSourceProgress::from_status(status);
    match (status.source.as_ref(), status.target.as_ref()) {
        (Some(source), Some(target)) => format!(
            "source_to_target_progress source_durable_lsn={} target_applied_lsn={} lag_bytes={} caught_up={}",
            source.last_durable_lsn,
            target.last_applied_lsn,
            progress.source_to_target_lag_bytes.unwrap_or_default(),
            progress.reaches_source_durable
        ),
        _ => "source_to_target_progress missing".to_string(),
    }
}

fn optional_bool_evidence(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}
