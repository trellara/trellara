use std::fmt::Write as _;

use crate::{FleetConvergenceGateStatus, FleetFlowSummary};

pub(crate) fn push_fleet_report_flow_text(output: &mut String, flow: &FleetFlowSummary) {
    writeln!(
        output,
        "- {} config={} mode={} stream={} target={} tables={}",
        flow.flow_id,
        flow.config,
        flow.mode,
        flow.stream_kind,
        flow.target_configured,
        flow.table_count
    )
    .expect("write string");
    writeln!(output, "  boundary: {}", flow.transaction_boundary).expect("write string");
    writeln!(
        output,
        "  lake_fanin: status={} mode={}",
        flow.lake_fanin.status.label(),
        flow.lake_fanin.fanin_mode
    )
    .expect("write string");
    writeln!(
        output,
        "    visibility: {}",
        flow.lake_fanin.epoch_visibility_boundary
    )
    .expect("write string");
    writeln!(
        output,
        "    iceberg_checkpoint_receipts: {}",
        flow.lake_fanin.iceberg_checkpoint_receipt_gate
    )
    .expect("write string");
    writeln!(
        output,
        "    watermarks: {}",
        flow.lake_fanin.source_watermark_rollup
    )
    .expect("write string");
    writeln!(
        output,
        "    straggler_policy: {}",
        flow.lake_fanin.straggler_policy
    )
    .expect("write string");
    for guidance in &flow.lake_fanin.guidance {
        writeln!(output, "    guidance: {guidance}").expect("write string");
    }
    writeln!(output, "  topics: {}", flow.topics.join(", ")).expect("write string");
    push_convergence_gates(output, flow);
    push_recovery_drills(output, flow);
    if !flow.risks.is_empty() {
        writeln!(output, "  risks: {}", flow.risks.join("; ")).expect("write string");
    }
}

fn push_convergence_gates(output: &mut String, flow: &FleetFlowSummary) {
    output.push_str("  convergence_gates:\n");
    for gate in &flow.convergence_gates {
        writeln!(
            output,
            "  - [{}] {}: {}",
            fleet_convergence_gate_status_label(gate.status),
            gate.code,
            gate.evidence
        )
        .expect("write string");
        writeln!(output, "    proof: {}", gate.proof_command).expect("write string");
    }
}

fn push_recovery_drills(output: &mut String, flow: &FleetFlowSummary) {
    output.push_str("  recovery_drills:\n");
    for drill in &flow.recovery_drills {
        writeln!(
            output,
            "  - {}: trigger={} goal={}",
            drill.code, drill.trigger, drill.operator_goal
        )
        .expect("write string");
        for command in &drill.commands {
            writeln!(output, "    command: {command}").expect("write string");
        }
        writeln!(output, "    success: {}", drill.success_evidence).expect("write string");
    }
}

fn fleet_convergence_gate_status_label(status: FleetConvergenceGateStatus) -> &'static str {
    match status {
        FleetConvergenceGateStatus::NeedsLiveEvidence => "needs_live_evidence",
        FleetConvergenceGateStatus::BlockedByConfig => "blocked_by_config",
    }
}
