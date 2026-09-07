use std::fmt::Write as _;

use crate::FleetScorecardSummary;

pub(crate) fn render_fleet_scorecard_text(summary: &FleetScorecardSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara fleet scorecard").expect("write string");
    writeln!(&mut output, "verdict: {}", summary.verdict).expect("write string");
    writeln!(&mut output, "score: {}", summary.score).expect("write string");
    writeln!(&mut output, "flows: {}", summary.flow_count).expect("write string");
    writeln!(
        &mut output,
        "flow_status: {} ready, {} review_required, {} blocked",
        summary.ready_flow_count, summary.review_required_flow_count, summary.blocked_flow_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "streams: local={} kafka={}",
        summary.local_stream_count, summary.kafka_stream_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "targets_configured: {}/{}",
        summary.target_configured_count, summary.flow_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "acceptance_gates: {} configuration_ready, {} needs_live_evidence, {} blocked",
        summary.configuration_ready_gate_count,
        summary.needs_live_evidence_gate_count,
        summary.blocked_gate_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "convergence_gates: {} total, {} blocked_by_config",
        summary.convergence_gate_count, summary.blocked_convergence_gate_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "recovery_drills: {}",
        summary.recovery_drill_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "topology_verdict: {}",
        summary.topology_verdict
    )
    .expect("write string");
    writeln!(
        &mut output,
        "analytical_fanin: verdict={} ready={} publishable_with_gaps={} blocked={}",
        summary.lake_fanin_verdict,
        summary.lake_ready_flow_count,
        summary.lake_publishable_with_gaps_flow_count,
        summary.lake_blocked_flow_count
    )
    .expect("write string");

    output.push_str("\nflows:\n");
    for flow in &summary.flows {
        writeln!(
            &mut output,
            "- {} config={} verdict={} score={} mode={} stream={} target={}",
            flow.flow_id,
            flow.config,
            flow.verdict,
            flow.score,
            flow.mode,
            flow.stream_kind,
            flow.target_configured
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  gates: {} total, {} needs_live_evidence, {} blocked",
            flow.gate_count, flow.needs_live_evidence_gate_count, flow.blocked_gate_count
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  convergence_gates: {} total, {} blocked_by_config",
            flow.convergence_gate_count, flow.blocked_convergence_gate_count
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  recovery_drills: {}",
            flow.recovery_drill_count
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  analytical_fanin: status={} mode={}",
            flow.lake_fanin_status, flow.lake_fanin_mode
        )
        .expect("write string");
        if !flow.risks.is_empty() {
            writeln!(&mut output, "  risks: {}", flow.risks.join("; ")).expect("write string");
        }
    }

    if !summary.blockers.is_empty() {
        output.push_str("\nblockers:\n");
        for blocker in &summary.blockers {
            writeln!(&mut output, "- {blocker}").expect("write string");
        }
    }

    if !summary.warnings.is_empty() {
        output.push_str("\nwarnings:\n");
        for warning in &summary.warnings {
            writeln!(&mut output, "- {warning}").expect("write string");
        }
    }

    output.push_str("\nreview_sequence:\n");
    for step in &summary.review_sequence {
        writeln!(&mut output, "- {step}").expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
