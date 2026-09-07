use crate::{
    source_postgres_risk_factors, source_slot_failover_factors, source_slot_issue_factors,
    source_wal_retention_factor, subscription_conflict_factors, FlowAlert, FlowAlertSeverity,
    FlowStatusSummary,
};

pub(crate) fn source_alerts(status: &FlowStatusSummary) -> Vec<FlowAlert> {
    let mut alerts = Vec::new();

    if !status.source_slot.exists {
        alerts.push(FlowAlert::critical(
            "source_slot_missing",
            format!(
                "source replication slot {} is missing",
                status.source_slot.slot_name
            ),
            "run trellara bootstrap to create or repair the source replication slot",
        ));
    } else if !status.source_slot.issues.is_empty() {
        for factor in source_slot_issue_factors(&status.source_slot) {
            alerts.push(match factor.severity {
                FlowAlertSeverity::Critical => {
                    FlowAlert::critical(factor.code, factor.evidence, factor.recommendation)
                }
                FlowAlertSeverity::Warning => {
                    FlowAlert::warning(factor.code, factor.evidence, factor.recommendation)
                }
            });
        }
    }
    for factor in source_slot_failover_factors(&status.source_slot) {
        alerts.push(FlowAlert::warning(
            factor.code,
            factor.evidence,
            factor.recommendation,
        ));
    }
    for factor in subscription_conflict_factors(&status.subscription_conflicts) {
        alerts.push(alert_from_factor(factor));
    }

    for factor in source_postgres_risk_factors(&status.source_slot) {
        alerts.push(alert_from_factor(factor));
    }
    if let Some(factor) = source_wal_retention_factor(
        &status.source_slot,
        status.source_wal_retention_warn_bytes,
        "drain relay/apply lag or reseed slow targets before source WAL retention grows further",
    ) {
        alerts.push(alert_from_factor(factor));
    }

    match status.source.as_ref() {
        Some(source) if !source.source_is_durable => alerts.push(FlowAlert::warning(
            "source_checkpoint_lag",
            format!(
                "source durable LSN {} is behind seen LSN {} by {} bytes",
                source.last_durable_lsn, source.last_seen_lsn, source.seen_to_durable_bytes
            ),
            "run trellara relay until source durable checkpoint catches up",
        )),
        None => alerts.push(FlowAlert::critical(
            "source_checkpoint_missing",
            "source checkpoint is missing",
            "run trellara relay after bootstrap to establish a source checkpoint",
        )),
        _ => {}
    }

    alerts
}

fn alert_from_factor(factor: crate::SourceSafetyFactor) -> FlowAlert {
    match factor.severity {
        FlowAlertSeverity::Critical => {
            FlowAlert::critical(factor.code, factor.evidence, factor.recommendation)
        }
        FlowAlertSeverity::Warning => {
            FlowAlert::warning(factor.code, factor.evidence, factor.recommendation)
        }
    }
}
