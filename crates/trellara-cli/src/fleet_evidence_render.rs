use std::fmt::Write as _;

use crate::FleetEvidencePlanSummary;

pub(crate) fn render_fleet_evidence_plan_text(summary: &FleetEvidencePlanSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara fleet evidence plan").expect("write string");
    writeln!(&mut output, "verdict: {}", summary.verdict).expect("write string");
    writeln!(
        &mut output,
        "flows: {} gates: {} needs_live_evidence: {} blocked: {} required_live_artifacts: {} commands: {}",
        summary.flow_count,
        summary.gate_count,
        summary.needs_live_evidence_gate_count,
        summary.blocked_gate_count,
        summary.required_live_evidence_artifact_count,
        summary.command_count
    )
    .expect("write string");
    writeln!(&mut output, "review_rule: {}", summary.review_rule).expect("write string");

    output.push_str("\nflows:\n");
    for flow in &summary.flows {
        writeln!(
            &mut output,
            "- {} config={} verdict={} needs_live_evidence={} blocked={} required_live_artifacts={}",
            flow.flow_id,
            flow.config,
            flow.verdict,
            flow.needs_live_evidence_gate_count,
            flow.blocked_gate_count,
            flow.required_live_evidence_artifact_count
        )
        .expect("write string");
        for gate in &flow.live_evidence_gates {
            writeln!(
                &mut output,
                "  - [needs_live_evidence] {} artifact={} command={}",
                gate.code, gate.artifact, gate.proof_command
            )
            .expect("write string");
            writeln!(
                &mut output,
                "    success_markers: {}",
                gate.success_markers.join(", ")
            )
            .expect("write string");
            writeln!(
                &mut output,
                "    requirements: {}",
                gate.collection_requirements.join("; ")
            )
            .expect("write string");
        }
        for gate in &flow.blocked_gates {
            writeln!(
                &mut output,
                "  - [blocked] {} artifact={} command={}",
                gate.code, gate.artifact, gate.proof_command
            )
            .expect("write string");
        }
    }

    output.push_str("\ncommand_sequence:\n");
    for command in &summary.command_sequence {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
