use crate::{
    validation_drift_message, validation_stale_message, FlowAlert, FlowStatusSummary,
    ValidationProgress,
};

pub(crate) fn failure_alerts(status: &FlowStatusSummary) -> Vec<FlowAlert> {
    let mut alerts = Vec::new();

    if let Some(quarantine) = status.latest_quarantine.as_ref() {
        alerts.push(FlowAlert::critical(
            "target_quarantine_blocked",
            format!(
                "target quarantine contains transaction {} at LSN {}: {}",
                quarantine.transaction_id, quarantine.commit_lsn, quarantine.reason
            ),
            "run trellara quarantine list, repair the target contract, then trellara quarantine replay-ready before redelivery",
        ));
    }

    if let Some(drift) = status.source_schema_drift.as_ref() {
        alerts.push(FlowAlert::critical(
            "source_schema_handoff_required",
            format!("{} for {}", drift.reason, drift.relations.join(", ")),
            drift.recommendation.clone(),
        ));
    }

    if let Some(validation) = status
        .latest_validation
        .as_ref()
        .filter(|validation| !validation.converged)
    {
        alerts.push(FlowAlert::warning(
            "validation_drift",
            validation_drift_message(validation),
            "run trellara verify after repair; use trellara reseed if checksum drift remains",
        ));
    }

    if let Some(validation) = status
        .latest_validation
        .as_ref()
        .filter(|validation| validation.converged)
    {
        let progress = ValidationProgress::from_parts(
            validation,
            status.source.as_ref(),
            status.target.as_ref(),
        );
        if !progress.is_current {
            alerts.push(FlowAlert::warning(
                "validation_stale",
                validation_stale_message(progress),
                "run trellara verify until validation watermarks reach the current source and target checkpoints",
            ));
        }
    }

    alerts
}
