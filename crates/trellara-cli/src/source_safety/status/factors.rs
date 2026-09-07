use crate::{
    status_alerts_failure::failure_alerts, status_alerts_partition::partition_alerts,
    status_alerts_target::target_alerts, FlowAlert, FlowAlertSeverity, FlowStatusSummary,
    SourceSafetyFactor,
};

pub(crate) fn status_target_factors(status: &FlowStatusSummary) -> Vec<SourceSafetyFactor> {
    alert_factors(target_alerts(status), FlowAlertSeverity::Warning, 10)
}

pub(crate) fn status_partition_factors(status: &FlowStatusSummary) -> Vec<SourceSafetyFactor> {
    alert_factors(partition_alerts(status), FlowAlertSeverity::Warning, 10)
}

pub(crate) fn status_recovery_factors(status: &FlowStatusSummary) -> Vec<SourceSafetyFactor> {
    failure_alerts(status)
        .into_iter()
        .filter_map(recovery_alert_factor)
        .collect()
}

fn recovery_alert_factor(alert: FlowAlert) -> Option<SourceSafetyFactor> {
    match alert.code.as_str() {
        "target_quarantine_blocked" => Some(alert_factor(alert, FlowAlertSeverity::Critical, 15)),
        "validation_drift" => Some(alert_factor(alert, FlowAlertSeverity::Warning, 10)),
        _ => None,
    }
}

fn alert_factors(
    alerts: Vec<FlowAlert>,
    severity: FlowAlertSeverity,
    points_lost: u8,
) -> Vec<SourceSafetyFactor> {
    alerts
        .into_iter()
        .map(|alert| alert_factor(alert, severity, points_lost))
        .collect()
}

fn alert_factor(
    alert: FlowAlert,
    severity: FlowAlertSeverity,
    points_lost: u8,
) -> SourceSafetyFactor {
    SourceSafetyFactor {
        code: alert.code,
        severity,
        points_lost,
        evidence: alert.message,
        recommendation: alert.recommendation,
    }
}
