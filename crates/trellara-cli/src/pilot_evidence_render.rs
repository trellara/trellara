use std::fmt::Write as _;

use crate::{
    PilotGuideOutputFormat, PilotLiveEvidenceCheckSummary, PilotLiveEvidenceStatus, Result,
};

pub(crate) fn render_pilot_evidence_check_summary(
    summary: &PilotLiveEvidenceCheckSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_pilot_evidence_check_text(summary)),
    }
}

pub(crate) fn render_pilot_evidence_check_text(summary: &PilotLiveEvidenceCheckSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara pilot evidence check").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "evidence_dir: {}", summary.evidence_dir).expect("write string");
    writeln!(&mut output, "verdict: {}", summary.verdict).expect("write string");
    writeln!(
        &mut output,
        "gates: {} accepted, {} missing, {} insufficient, {} not_required, {} blocked",
        summary.accepted_gate_count,
        summary.missing_gate_count,
        summary.insufficient_gate_count,
        summary.not_required_gate_count,
        summary.blocked_gate_count
    )
    .expect("write string");
    writeln!(&mut output, "review_rule: {}", summary.review_rule).expect("write string");

    output.push_str("\ngates:\n");
    for gate in &summary.gates {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            pilot_live_evidence_status_label(gate.evidence_status),
            gate.code,
            gate.title
        )
        .expect("write string");
        if let Some(artifact) = &gate.artifact {
            writeln!(&mut output, "  artifact: {artifact}").expect("write string");
        }
        writeln!(&mut output, "  proof: {}", gate.proof_command).expect("write string");
        writeln!(&mut output, "  note: {}", gate.note).expect("write string");
        if !gate.observed_markers.is_empty() {
            writeln!(
                &mut output,
                "  observed_markers: {}",
                gate.observed_markers.join(", ")
            )
            .expect("write string");
        }
        if !gate.missing_markers.is_empty() {
            writeln!(
                &mut output,
                "  missing_markers: {}",
                gate.missing_markers.join(", ")
            )
            .expect("write string");
        }
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}

fn pilot_live_evidence_status_label(status: PilotLiveEvidenceStatus) -> &'static str {
    match status {
        PilotLiveEvidenceStatus::Accepted => "accepted",
        PilotLiveEvidenceStatus::Missing => "missing",
        PilotLiveEvidenceStatus::Insufficient => "insufficient",
        PilotLiveEvidenceStatus::NotRequired => "not_required",
        PilotLiveEvidenceStatus::Blocked => "blocked",
    }
}
