use crate::{
    push_checkpoint_lag_metrics, push_checkpoint_metrics, push_checksum_status_metric,
    push_flow_readiness_metrics, push_health_status_metric, push_partition_watermark_metrics,
    push_proof_check_metrics, push_quarantine_metrics, push_source_stream_metrics,
    push_transaction_boundary_metrics, push_validation_freshness_metrics, CorrectnessReportSummary,
    FlowAlertsSummary, FlowStatusSummary, QUICKSTART_ESTIMATED_MINUTES,
    QUICKSTART_TIME_BUDGET_MINUTES,
};

pub(crate) use crate::status_metric_format::{
    bool_value, checksum_status_label, correctness_proof_status_label, flow_health_status_label,
    push_metric, transaction_boundary_status_label,
};

pub(crate) fn render_prometheus_metrics(status: FlowStatusSummary) -> String {
    let report = CorrectnessReportSummary::from_status(status.clone());
    let alerts = FlowAlertsSummary::from_status(status.clone());
    let base_labels = [
        ("source_id", status.source_id.as_str()),
        ("dataset_id", status.dataset_id.as_str()),
        ("mode", status.mode.as_str()),
    ];
    let mut output = String::new();

    push_flow_readiness_metrics(&mut output, &base_labels, &status, &report, &alerts);
    push_metric(
        &mut output,
        "trellara_quickstart_estimated_minutes",
        &base_labels,
        QUICKSTART_ESTIMATED_MINUTES,
    );
    push_metric(
        &mut output,
        "trellara_quickstart_time_budget_minutes",
        &base_labels,
        QUICKSTART_TIME_BUDGET_MINUTES,
    );
    push_quarantine_metrics(&mut output, &status);
    push_health_status_metric(&mut output, &status);
    push_checksum_status_metric(&mut output, &status, &report);
    push_validation_freshness_metrics(&mut output, &status, &report);

    push_transaction_boundary_metrics(&mut output, &base_labels, &status, &report);

    push_checkpoint_metrics(&mut output, &base_labels, &status);
    push_source_stream_metrics(&mut output, &base_labels, &status);
    push_checkpoint_lag_metrics(&mut output, &base_labels, &status);
    push_partition_watermark_metrics(&mut output, &base_labels, &status);

    push_proof_check_metrics(&mut output, &base_labels, &status, &report);

    output
}
