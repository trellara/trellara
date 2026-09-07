use std::fmt::Write as _;

use crate::{
    checksum_status_label, correctness_proof_status_label, transaction_boundary_status_label,
    CorrectnessReportSummary,
};

pub(crate) fn render_correctness_report_text(summary: &CorrectnessReportSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara correctness report").expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "mode: {}", summary.mode).expect("write string");
    writeln!(&mut output, "ready: {}", summary.ready).expect("write string");
    writeln!(
        &mut output,
        "proof_checks: {}/{} verified, {} at_risk, {} missing_evidence",
        summary.verified_proof_check_count,
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
        "transaction_boundary_evidence: {}",
        summary.transaction_boundary.evidence
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transaction_visibility_contract: {}",
        summary.transaction_boundary.visibility_contract
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
        checksum_status_label(summary.latest_checksum_status)
    )
    .expect("write string");

    output.push_str("\nproof_checks:\n");
    for check in &summary.proof_checks {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            correctness_proof_status_label(check.status),
            check.code,
            check.evidence
        )
        .expect("write string");
        if let Some(issue_code) = &check.issue_code {
            writeln!(&mut output, "  issue_code: {issue_code}").expect("write string");
        }
        if let Some(recommendation) = &check.recommendation {
            writeln!(&mut output, "  recommendation: {recommendation}").expect("write string");
        }
    }

    output.push_str("\nrecommended_actions:\n");
    if summary.recommended_actions.is_empty() {
        output.push_str("- none\n");
    } else {
        for action in &summary.recommended_actions {
            writeln!(&mut output, "- {action}").expect("write string");
        }
    }
    output
}
