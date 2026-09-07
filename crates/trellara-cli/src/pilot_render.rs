use std::fmt::Write as _;

use crate::{
    PilotGuideOutputFormat, PilotGuideSummary, PilotScorecardStatus, PilotScorecardSummary, Result,
};

pub(crate) fn render_pilot_guide_summary(
    summary: &PilotGuideSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_pilot_guide_text(summary)),
    }
}

pub(crate) fn render_pilot_scorecard_summary(
    summary: &PilotScorecardSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_pilot_scorecard_text(summary)),
    }
}

pub(crate) fn render_pilot_guide_text(summary: &PilotGuideSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara pilot guide").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "objective: {}", summary.objective).expect("write string");
    writeln!(
        &mut output,
        "time_budget: {} minutes",
        summary.evaluation_time_budget_minutes
    )
    .expect("write string");
    writeln!(&mut output, "stream: {}", summary.stream_kind).expect("write string");
    writeln!(&mut output, "kafka_required: {}", summary.kafka_required).expect("write string");
    writeln!(
        &mut output,
        "transaction_boundary: {}",
        summary.transaction_boundary
    )
    .expect("write string");
    writeln!(
        &mut output,
        "capture_spill_boundary: {}",
        summary.capture_spill_boundary
    )
    .expect("write string");
    writeln!(&mut output, "tables: {}", summary.tables.join(", ")).expect("write string");

    output.push_str("\nphases:\n");
    for phase in &summary.phases {
        writeln!(&mut output, "{}. {}", phase.order, phase.name).expect("write string");
        writeln!(&mut output, "   command: {}", phase.command).expect("write string");
        writeln!(&mut output, "   proof: {}", phase.proof).expect("write string");
    }

    output.push_str("\nevidence_commands:\n");
    for command in &summary.evidence_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nlarge_transaction_evidence:\n");
    for command in &summary.large_transaction_evidence {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nfailure_drill:\n");
    for command in &summary.failure_drill {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nacceptance_gates:\n");
    for gate in &summary.acceptance_gates {
        writeln!(&mut output, "- {gate}").expect("write string");
    }

    output
}

pub(crate) fn render_pilot_scorecard_text(summary: &PilotScorecardSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara pilot scorecard").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "verdict: {}", summary.verdict).expect("write string");
    writeln!(&mut output, "score: {}", summary.score).expect("write string");
    writeln!(
        &mut output,
        "gates: {} configuration_ready, {} needs_live_evidence, {} blocked",
        summary.configuration_ready_gate_count,
        summary.needs_live_evidence_gate_count,
        summary.blocked_gate_count
    )
    .expect("write string");

    output.push_str("\nacceptance_gates:\n");
    for gate in &summary.gates {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            pilot_scorecard_status_label(gate.status),
            gate.code,
            gate.name
        )
        .expect("write string");
        writeln!(&mut output, "  evidence: {}", gate.evidence).expect("write string");
        writeln!(&mut output, "  proof: {}", gate.proof_command).expect("write string");
        writeln!(&mut output, "  acceptance: {}", gate.acceptance).expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}

pub(crate) fn pilot_scorecard_status_label(status: PilotScorecardStatus) -> &'static str {
    match status {
        PilotScorecardStatus::ConfigurationReady => "configuration_ready",
        PilotScorecardStatus::NeedsLiveEvidence => "needs_live_evidence",
        PilotScorecardStatus::EnvironmentSpecific => "environment_specific",
        PilotScorecardStatus::Blocked => "blocked",
    }
}
