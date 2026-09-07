use std::fmt::Write as _;

use crate::{EnterpriseEvaluationSummary, PilotGuideOutputFormat, Result};

pub(crate) fn render_enterprise_evaluation_summary(
    summary: &EnterpriseEvaluationSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_enterprise_evaluation_text(summary)),
    }
}

pub(crate) fn render_enterprise_evaluation_text(summary: &EnterpriseEvaluationSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara enterprise evaluation").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "verdict: {}", summary.verdict).expect("write string");
    writeln!(
        &mut output,
        "recommended_mode: {}",
        summary.recommended_mode
    )
    .expect("write string");
    writeln!(&mut output, "buyer_summary: {}", summary.buyer_summary).expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "mode_contract: {}", summary.mode_contract).expect("write string");
    writeln!(
        &mut output,
        "transaction_boundary_contract: {}",
        summary.transaction_boundary_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "scorecard_gates: {} configuration_ready, {} needs_live_evidence, {} blocked",
        summary.configuration_ready_gate_count,
        summary.needs_live_evidence_gate_count,
        summary.blocked_gate_count
    )
    .expect("write string");

    output.push_str("\npublic_modes:\n");
    for mode in &summary.public_modes {
        writeln!(&mut output, "- {mode}").expect("write string");
    }

    output.push_str("\ndifferentiators:\n");
    for differentiator in &summary.differentiators {
        writeln!(&mut output, "- {differentiator}").expect("write string");
    }

    output.push_str("\nreadiness_gates:\n");
    for gate in &summary.readiness_gates {
        writeln!(
            &mut output,
            "- {} artifact={} command={} pass_condition={}",
            gate.area, gate.artifact, gate.command, gate.pass_condition
        )
        .expect("write string");
    }

    output.push_str("\nproof_commands:\n");
    for command in &summary.proof_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nlive_evidence_required:\n");
    for evidence in &summary.live_evidence_required {
        writeln!(&mut output, "- {evidence}").expect("write string");
    }

    if !summary.blockers.is_empty() {
        output.push_str("\nblockers:\n");
        for blocker in &summary.blockers {
            writeln!(&mut output, "- {blocker}").expect("write string");
        }
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
