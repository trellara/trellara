use std::fmt::Write as _;

use crate::{
    checksum_status_label, correctness_proof_status_label, flow_health_status_label,
    transaction_boundary_status_label, DashboardSummary,
};

pub(crate) fn render_dashboard_text(summary: &DashboardSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara dashboard").expect("write string");
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
    writeln!(
        &mut output,
        "quickstart: estimated {} minutes, budget {} minutes",
        summary.quickstart_estimated_minutes, summary.quickstart_time_budget_minutes
    )
    .expect("write string");
    writeln!(
        &mut output,
        "proof_checks: {} total, {} at_risk, {} missing_evidence",
        summary.proof_check_count,
        summary.at_risk_proof_check_count,
        summary.missing_evidence_proof_check_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transaction_boundary: {}",
        transaction_boundary_status_label(summary.transaction_boundary.status)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "parallel_replay_contract: {}",
        summary.transaction_boundary.parallel_replay_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "checksum_status: {}",
        checksum_status_label(summary.checksum_status)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "snapshot_handoff: {}",
        correctness_proof_status_label(summary.snapshot_handoff_status)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "snapshot_handoff_evidence: {}",
        summary
            .snapshot_handoff_evidence
            .as_deref()
            .unwrap_or("missing")
    )
    .expect("write string");
    output.push_str("\nwatermarks:\n");
    writeln!(
        &mut output,
        "- source: {}",
        summary.source_watermark_lsn.as_deref().unwrap_or("missing")
    )
    .expect("write string");
    writeln!(
        &mut output,
        "- target: {}",
        summary.target_watermark_lsn.as_deref().unwrap_or("missing")
    )
    .expect("write string");
    writeln!(
        &mut output,
        "- source_to_target_lag_bytes: {}",
        summary
            .source_to_target_lag_bytes
            .map(|lag| lag.to_string())
            .unwrap_or_else(|| "missing".to_string())
    )
    .expect("write string");
    writeln!(
        &mut output,
        "- partition_global_applied: {}",
        summary
            .partition_global_applied_lsn
            .as_deref()
            .unwrap_or("not_applicable")
    )
    .expect("write string");
    output
}
