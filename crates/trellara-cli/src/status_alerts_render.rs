use std::fmt::Write as _;

use crate::{
    flow_health_status_label, source_safety::labels::flow_alert_severity_label, FlowAlertsSummary,
};

pub(crate) fn render_flow_alerts_text(summary: &FlowAlertsSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara flow alerts").expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(
        &mut output,
        "status: {}",
        flow_health_status_label(summary.status)
    )
    .expect("write string");
    writeln!(&mut output, "alerts: {}", summary.alert_count).expect("write string");
    writeln!(
        &mut output,
        "highest_severity: {}",
        summary
            .highest_severity
            .map(flow_alert_severity_label)
            .unwrap_or("none")
    )
    .expect("write string");
    output.push_str("\nalerts:\n");
    if summary.alerts.is_empty() {
        output.push_str("- none\n");
    } else {
        for alert in &summary.alerts {
            writeln!(
                &mut output,
                "- [{}] {}: {}",
                flow_alert_severity_label(alert.severity),
                alert.code,
                alert.message
            )
            .expect("write string");
            writeln!(&mut output, "  recommendation: {}", alert.recommendation)
                .expect("write string");
        }
    }
    output
}
