use crate::status_metric_format::push_metric;
use crate::{CorrectnessReportSummary, FlowStatusSummary};

pub(crate) fn push_proof_check_metrics(
    output: &mut String,
    base_labels: &[(&str, &str)],
    status: &FlowStatusSummary,
    report: &CorrectnessReportSummary,
) {
    push_metric(
        output,
        "trellara_proof_checks_total",
        base_labels,
        report.proof_check_count,
    );
    for (status_label, value) in [
        ("verified", report.verified_proof_check_count),
        ("at_risk", report.at_risk_proof_check_count),
        (
            "missing_evidence",
            report.missing_evidence_proof_check_count,
        ),
    ] {
        let labels = [
            ("source_id", status.source_id.as_str()),
            ("dataset_id", status.dataset_id.as_str()),
            ("mode", status.mode.as_str()),
            ("status", status_label),
        ];
        push_metric(
            output,
            "trellara_proof_checks_by_status_total",
            &labels,
            value,
        );
    }
}
