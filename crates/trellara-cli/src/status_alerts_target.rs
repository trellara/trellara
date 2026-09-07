use crate::{FlowAlert, FlowStatusSummary, TargetSourceProgress};

pub(crate) fn target_alerts(status: &FlowStatusSummary) -> Vec<FlowAlert> {
    let progress = TargetSourceProgress::from_status(status);
    match (status.source.as_ref(), status.target.as_ref()) {
        (Some(source), Some(target))
            if target.target_is_caught_up && !progress.reaches_source_durable =>
        {
            vec![FlowAlert::warning(
                "target_source_watermark_lag",
                format!(
                    "target applied LSN {} is behind source durable LSN {} by {} bytes",
                    target.last_applied_lsn,
                    source.last_durable_lsn,
                    progress.source_to_target_lag_bytes.unwrap_or_default()
                ),
                "run trellara relay and trellara apply until target applied watermark reaches the source durable watermark",
            )]
        }
        (Some(_source), Some(target)) if !target.target_is_caught_up => vec![FlowAlert::warning(
            "target_checkpoint_lag",
            format!(
                "target applied LSN {} is behind durable LSN {} by {} bytes",
                target.last_applied_lsn, target.last_durable_lsn, target.durable_to_applied_bytes
            ),
            "run trellara apply until target applied watermark catches up",
        )],
        (_, None) => vec![FlowAlert::critical(
            "target_checkpoint_missing",
            "target checkpoint is missing",
            "run trellara apply-schema and trellara apply to establish a target checkpoint",
        )],
        _ => Vec::new(),
    }
}
