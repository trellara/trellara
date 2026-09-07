use crate::{
    validation_drift_message, validation_stale_message, FlowFailureParts, FlowFailureSummary,
    TargetSourceProgress, ValidationProgress,
};

pub(crate) fn target_failure_from_parts(
    parts: &FlowFailureParts<'_>,
    recovery_action_codes: &[String],
) -> Option<FlowFailureSummary> {
    if let Some(validation) = parts
        .latest_validation
        .filter(|validation| !validation.converged)
    {
        return Some(FlowFailureSummary::warning(
            "validation_drift",
            validation_drift_message(validation),
            Some(validation.completed_at.clone()),
            recovery_action_codes.to_vec(),
        ));
    }
    if let Some(validation) = parts
        .latest_validation
        .filter(|validation| validation.converged)
    {
        let progress = ValidationProgress::from_parts(validation, parts.source, parts.target);
        if !progress.is_current {
            return Some(FlowFailureSummary::warning(
                "validation_stale",
                validation_stale_message(progress),
                Some(validation.completed_at.clone()),
                recovery_action_codes.to_vec(),
            ));
        }
    }
    if let Some(target) = parts.target.filter(|target| !target.target_is_caught_up) {
        return Some(FlowFailureSummary::warning(
            "target_checkpoint_lag",
            format!(
                "target applied LSN {} is behind durable LSN {} by {} bytes",
                target.last_applied_lsn, target.last_durable_lsn, target.durable_to_applied_bytes
            ),
            None,
            recovery_action_codes.to_vec(),
        ));
    }
    let progress = TargetSourceProgress::from_lags(parts.source, parts.target);
    if let (Some(source), Some(target)) = (parts.source, parts.target) {
        if target.target_is_caught_up && !progress.reaches_source_durable {
            return Some(FlowFailureSummary::warning(
                "target_source_watermark_lag",
                format!(
                    "target applied LSN {} is behind source durable LSN {} by {} bytes",
                    target.last_applied_lsn,
                    source.last_durable_lsn,
                    progress.source_to_target_lag_bytes.unwrap_or_default()
                ),
                None,
                recovery_action_codes.to_vec(),
            ));
        }
    }
    if let Some(watermarks) = parts.partition_watermarks {
        if !watermarks.complete_partition_set {
            return Some(FlowFailureSummary::warning(
                "partition_watermark_incomplete",
                format!(
                    "partition watermarks are missing {} of {} partitions",
                    watermarks.missing_partitions.len(),
                    watermarks.expected_partition_count
                ),
                None,
                recovery_action_codes.to_vec(),
            ));
        }
        if watermarks
            .global_durable_to_applied_bytes
            .is_some_and(|lag| lag > 0)
        {
            return Some(FlowFailureSummary::warning(
                "partition_global_watermark_lag",
                format!(
                    "partition global applied watermark {} is behind durable watermark {} by {} bytes",
                    watermarks.global_applied_lsn.as_deref().unwrap_or(""),
                    watermarks.global_durable_lsn.as_deref().unwrap_or(""),
                    watermarks.global_durable_to_applied_bytes.unwrap_or_default()
                ),
                None,
                recovery_action_codes.to_vec(),
            ));
        }
    }

    None
}
