use std::path::{Path, PathBuf};

use crate::{
    FleetConvergenceGateStatus, FleetFlowSummary, FleetReportSummary, FleetScorecardFlow,
    PilotScorecardSummary, TrellaraConfig,
};

pub(crate) fn fleet_scorecard_flow(
    config: &TrellaraConfig,
    config_path: &Path,
    flow_report: &FleetFlowSummary,
) -> FleetScorecardFlow {
    let scorecard = PilotScorecardSummary::from_config(config, config_path);
    let blocked_convergence_gate_count = blocked_convergence_gate_count(flow_report);

    FleetScorecardFlow {
        flow_id: flow_report.flow_id.clone(),
        config: flow_report.config.clone(),
        mode: flow_report.mode.clone(),
        stream_kind: flow_report.stream_kind.clone(),
        target_configured: flow_report.target_configured,
        verdict: scorecard.verdict,
        score: scorecard.score,
        gate_count: scorecard.gate_count,
        needs_live_evidence_gate_count: scorecard.needs_live_evidence_gate_count,
        blocked_gate_count: scorecard.blocked_gate_count,
        convergence_gate_count: flow_report.convergence_gates.len(),
        blocked_convergence_gate_count,
        recovery_drill_count: flow_report.recovery_drills.len(),
        lake_fanin_status: flow_report.lake_fanin.status.label().to_string(),
        lake_fanin_mode: flow_report.lake_fanin.fanin_mode.clone(),
        risks: flow_report.risks.clone(),
    }
}

pub(crate) fn fleet_scorecard_blockers(flows: &[FleetScorecardFlow]) -> Vec<String> {
    let mut blockers = Vec::new();
    for flow in flows {
        if flow.blocked_gate_count > 0 {
            blockers.push(format!(
                "{} has {} blocked pilot gate(s)",
                flow.flow_id, flow.blocked_gate_count
            ));
        }
        if flow.blocked_convergence_gate_count > 0 {
            blockers.push(format!(
                "{} has {} convergence gate(s) blocked by config",
                flow.flow_id, flow.blocked_convergence_gate_count
            ));
        }
    }
    blockers
}

pub(crate) fn ready_flow_count(flows: &[FleetScorecardFlow]) -> usize {
    flows
        .iter()
        .filter(|flow| {
            flow.blocked_gate_count == 0
                && flow.blocked_convergence_gate_count == 0
                && flow.risks.is_empty()
        })
        .count()
}

pub(crate) fn blocked_flow_count(flows: &[FleetScorecardFlow]) -> usize {
    flows
        .iter()
        .filter(|flow| flow.blocked_gate_count > 0 || flow.blocked_convergence_gate_count > 0)
        .count()
}

pub(crate) fn configuration_ready_gate_count(flows: &[FleetScorecardFlow]) -> usize {
    flows
        .iter()
        .map(|flow| flow.gate_count - flow.needs_live_evidence_gate_count - flow.blocked_gate_count)
        .sum()
}

pub(crate) fn fleet_score(flows: &[FleetScorecardFlow]) -> u8 {
    flows
        .iter()
        .map(|flow| flow.score as usize)
        .sum::<usize>()
        .checked_div(flows.len())
        .unwrap_or(0) as u8
}

pub(crate) fn fleet_scorecard_verdict(
    blocked_flow_count: usize,
    report: &FleetReportSummary,
) -> String {
    if blocked_flow_count > 0 || report.blocked_convergence_gate_count > 0 {
        "blocked_before_fleet_pilot"
    } else if !report.warnings.is_empty() {
        "review_required_before_fleet_pilot"
    } else {
        "ready_for_design_partner_fleet_review"
    }
    .to_string()
}

pub(crate) fn fleet_scorecard_next_commands(
    configs: &[(TrellaraConfig, PathBuf)],
    report: &FleetReportSummary,
) -> Vec<String> {
    let mut next_commands = vec![format!(
        "trellara fleet report {} --format text",
        configs
            .iter()
            .map(|(_, path)| format!("--config {}", path.display()))
            .collect::<Vec<_>>()
            .join(" ")
    )];
    next_commands.extend(report.next_commands.clone());
    next_commands
}

pub(crate) fn fleet_scorecard_review_sequence() -> Vec<String> {
    vec![
        "run source-safety and contract-test for every flow".to_string(),
        "prove one brokerless local flow end to end before widening the fleet".to_string(),
        "compare fleet report and fleet scorecard before design-partner review".to_string(),
        "rehearse target quarantine replay, checksum reseed, source WAL-loss reseed, and schema handoff refresh".to_string(),
    ]
}

fn blocked_convergence_gate_count(flow_report: &FleetFlowSummary) -> usize {
    flow_report
        .convergence_gates
        .iter()
        .filter(|gate| gate.status == FleetConvergenceGateStatus::BlockedByConfig)
        .count()
}
