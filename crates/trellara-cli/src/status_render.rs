use std::fmt::Write as _;

use crate::{flow_health_status_label, FlowStatusSummary, TargetSourceProgress};

pub(crate) fn render_flow_status_text(summary: &FlowStatusSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara flow status").expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "mode: {}", summary.mode).expect("write string");
    writeln!(
        &mut output,
        "status: {}",
        flow_health_status_label(summary.health.status)
    )
    .expect("write string");
    writeln!(&mut output, "issues: {}", summary.health.issue_count).expect("write string");
    if !summary.health.issues.is_empty() {
        output.push_str("\nissues:\n");
        for issue in &summary.health.issues {
            writeln!(&mut output, "- {issue}").expect("write string");
        }
    }
    output.push_str("\nwatermarks:\n");
    writeln!(
        &mut output,
        "- source_durable_lsn: {}",
        summary
            .source
            .as_ref()
            .map(|source| source.last_durable_lsn.as_str())
            .unwrap_or("missing")
    )
    .expect("write string");
    writeln!(
        &mut output,
        "- target_applied_lsn: {}",
        summary
            .target
            .as_ref()
            .map(|target| target.last_applied_lsn.as_str())
            .unwrap_or("missing")
    )
    .expect("write string");
    writeln!(
        &mut output,
        "- source_to_target_lag_bytes: {}",
        TargetSourceProgress::from_status(summary)
            .source_to_target_lag_bytes
            .map(|lag| lag.to_string())
            .unwrap_or_else(|| "missing".to_string())
    )
    .expect("write string");
    writeln!(
        &mut output,
        "- partition_global_applied_lsn: {}",
        summary
            .partition_watermarks
            .as_ref()
            .and_then(|watermarks| watermarks.global_applied_lsn.as_deref())
            .unwrap_or("not_applicable")
    )
    .expect("write string");
    output
}
