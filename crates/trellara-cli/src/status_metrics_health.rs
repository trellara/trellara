use crate::status_metric_format::{
    bool_value, checksum_status_label, flow_health_status_label, push_metric,
};
use crate::{CorrectnessReportSummary, FlowAlertsSummary, FlowStatusSummary};

pub(crate) fn push_flow_readiness_metrics(
    output: &mut String,
    base_labels: &[(&str, &str)],
    status: &FlowStatusSummary,
    report: &CorrectnessReportSummary,
    alerts: &FlowAlertsSummary,
) {
    push_metric(
        output,
        "trellara_flow_ready",
        base_labels,
        bool_value(report.ready),
    );
    push_metric(
        output,
        "trellara_flow_issues_total",
        base_labels,
        status.health.issue_count,
    );
    push_metric(
        output,
        "trellara_flow_recovery_actions_total",
        base_labels,
        status.recovery_actions.len(),
    );
    push_metric(
        output,
        "trellara_flow_alerts_total",
        base_labels,
        alerts.alert_count,
    );
}

pub(crate) fn push_quarantine_metrics(output: &mut String, status: &FlowStatusSummary) {
    if let Some(quarantine) = status.latest_quarantine.as_ref() {
        let quarantine_labels = [
            ("source_id", status.source_id.as_str()),
            ("dataset_id", status.dataset_id.as_str()),
            ("mode", status.mode.as_str()),
            ("reason", quarantine.reason.as_str()),
        ];
        push_metric(
            output,
            "trellara_target_quarantine_blocked",
            &quarantine_labels,
            1,
        );
    }
}

pub(crate) fn push_health_status_metric(output: &mut String, status: &FlowStatusSummary) {
    let health_status = flow_health_status_label(status.health.status);
    let health_labels = [
        ("source_id", status.source_id.as_str()),
        ("dataset_id", status.dataset_id.as_str()),
        ("mode", status.mode.as_str()),
        ("status", health_status),
    ];
    push_metric(output, "trellara_flow_health_status", &health_labels, 1);
}

pub(crate) fn push_checksum_status_metric(
    output: &mut String,
    status: &FlowStatusSummary,
    report: &CorrectnessReportSummary,
) {
    let checksum_status = checksum_status_label(report.latest_checksum_status);
    let checksum_labels = [
        ("source_id", status.source_id.as_str()),
        ("dataset_id", status.dataset_id.as_str()),
        ("mode", status.mode.as_str()),
        ("status", checksum_status),
    ];
    push_metric(output, "trellara_checksum_status", &checksum_labels, 1);
}

pub(crate) fn push_validation_freshness_metrics(
    output: &mut String,
    status: &FlowStatusSummary,
    report: &CorrectnessReportSummary,
) {
    let validation_labels = [
        ("source_id", status.source_id.as_str()),
        ("dataset_id", status.dataset_id.as_str()),
        ("mode", status.mode.as_str()),
    ];
    push_metric(
        output,
        "trellara_validation_current",
        &validation_labels,
        bool_value(report.latest_validation_current),
    );
    if let Some(lag) = report.latest_validation_source_lag_bytes {
        push_metric(
            output,
            "trellara_validation_source_lag_bytes",
            &validation_labels,
            lag,
        );
    }
    if let Some(lag) = report.latest_validation_target_lag_bytes {
        push_metric(
            output,
            "trellara_validation_target_lag_bytes",
            &validation_labels,
            lag,
        );
    }
}
