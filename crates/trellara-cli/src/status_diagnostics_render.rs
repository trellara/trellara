use std::fmt::Write as _;

use std::path::Path;

use crate::{
    flow_health_status_label, CorrectnessReportSummary, DiagnosticsBundleSummary,
    FlowAlertsSummary, FlowStatusSummary, RepairPlanSummary, TransactionBoundaryStatus,
    TransactionBoundarySummary,
};

impl DiagnosticsBundleSummary {
    pub(crate) fn from_status(status: FlowStatusSummary, config_path: &Path) -> Self {
        let report = CorrectnessReportSummary::from_status(status.clone());
        let alerts = FlowAlertsSummary::from_status(status.clone());
        let repair_plan = RepairPlanSummary::from_status(status.clone(), config_path);
        let barrier_blockers = barrier_blockers(&report.transaction_boundary);
        let metrics = crate::render_prometheus_metrics(status.clone());
        let config = config_path.display().to_string();
        let health_status = status.health.status;

        Self {
            source_id: status.source_id,
            dataset_id: status.dataset_id,
            mode: status.mode,
            config: config.clone(),
            ready: report.ready,
            status: health_status,
            latest_failure: status.latest_failure,
            report,
            alerts,
            repair_plan,
            barrier_blockers,
            metrics,
            attachment_commands: vec![
                format!("trellara status --config {config} --view report --format text"),
                format!("trellara status --config {config} --view dashboard --format text"),
                format!("trellara status --config {config} --view metrics"),
                format!("trellara status --config {config} --view diagnostics --format text"),
                format!("trellara repair-plan --config {config}"),
                format!("trellara quarantine list --config {config}"),
            ],
        }
    }
}

pub(crate) fn render_diagnostics_text(summary: &DiagnosticsBundleSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara diagnostics").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "mode: {}", summary.mode).expect("write string");
    writeln!(&mut output, "ready: {}", summary.ready).expect("write string");
    writeln!(
        &mut output,
        "status: {}",
        flow_health_status_label(summary.status)
    )
    .expect("write string");
    writeln!(&mut output, "alerts: {}", summary.alerts.alert_count).expect("write string");
    writeln!(
        &mut output,
        "repair_plan_required: {}",
        summary.repair_plan.plan_required
    )
    .expect("write string");
    output.push_str("\nbarrier_blockers:\n");
    if summary.barrier_blockers.is_empty() {
        output.push_str("- none\n");
    } else {
        for blocker in &summary.barrier_blockers {
            writeln!(&mut output, "- {blocker}").expect("write string");
        }
    }
    output.push_str("\nattachment_commands:\n");
    for command in &summary.attachment_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }
    output.push_str("\nrecommended_actions:\n");
    if summary.report.recommended_actions.is_empty() {
        output.push_str("- none\n");
    } else {
        for action in &summary.report.recommended_actions {
            writeln!(&mut output, "- {action}").expect("write string");
        }
    }
    output
}

fn barrier_blockers(boundary: &TransactionBoundarySummary) -> Vec<String> {
    if !boundary.manifest_barrier_required || boundary.status == TransactionBoundaryStatus::Verified
    {
        return Vec::new();
    }

    let mut blockers = Vec::new();
    if boundary.manifest_barrier_complete != Some(true) {
        blockers.push(format!(
            "{} manifest barrier is not complete",
            boundary.mode
        ));
    }
    if boundary.global_partition_watermark_caught_up == Some(false) {
        blockers.push("global partition watermark has not caught up".to_string());
    }
    blockers
}
