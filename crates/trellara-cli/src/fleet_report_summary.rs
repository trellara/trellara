use std::collections::BTreeSet;

use crate::fleet_types::*;
use crate::fleet_warnings::fleet_warnings;
use crate::FleetLakeFaninStatus;

impl FleetReportSummary {
    pub(crate) fn from_flows(flows: Vec<FleetFlowSummary>) -> Self {
        let flow_count = flows.len();
        let source_count = flows
            .iter()
            .map(|flow| flow.source_id.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        let dataset_count = flows
            .iter()
            .map(|flow| flow.dataset_id.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        let table_count = flows.iter().map(|flow| flow.table_count).sum();
        let target_configured_count = flows.iter().filter(|flow| flow.target_configured).count();
        let local_stream_count = flows
            .iter()
            .filter(|flow| flow.stream_kind == "local")
            .count();
        let kafka_stream_count = flows
            .iter()
            .filter(|flow| flow.stream_kind == "kafka")
            .count();
        let partitioned_flow_count = flows
            .iter()
            .filter(|flow| flow.mode == "partitioned_scale_mode")
            .count();
        let strict_chunked_flow_count = flows
            .iter()
            .filter(|flow| flow.mode == "strict_chunked_transaction_order")
            .count();
        let convergence_gate_count = flows.iter().map(|flow| flow.convergence_gates.len()).sum();
        let blocked_convergence_gate_count = flows
            .iter()
            .flat_map(|flow| flow.convergence_gates.iter())
            .filter(|gate| gate.status == FleetConvergenceGateStatus::BlockedByConfig)
            .count();
        let recovery_drill_count = flows.iter().map(|flow| flow.recovery_drills.len()).sum();
        let lake_ready_flow_count = flows
            .iter()
            .filter(|flow| flow.lake_fanin.status == FleetLakeFaninStatus::Ready)
            .count();
        let lake_publishable_with_gaps_flow_count = flows
            .iter()
            .filter(|flow| flow.lake_fanin.status == FleetLakeFaninStatus::PublishableWithGaps)
            .count();
        let lake_blocked_flow_count = flows
            .iter()
            .filter(|flow| flow.lake_fanin.status == FleetLakeFaninStatus::Blocked)
            .count();
        let lake_fanin_verdict = if lake_blocked_flow_count > 0 {
            "blocked".to_string()
        } else if lake_publishable_with_gaps_flow_count > 0 {
            "publishable_with_gaps".to_string()
        } else {
            "ready".to_string()
        };
        let warnings = fleet_warnings(&flows);
        let topology_verdict = if blocked_convergence_gate_count > 0 {
            "needs_targets"
        } else if !warnings.is_empty() {
            "review_required"
        } else {
            "ready_for_design_partner_review"
        }
        .to_string();
        let proof_commands = flows
            .iter()
            .flat_map(|flow| flow.proof_commands.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let next_commands = vec![
            "run trellara-check for each source before creating slots".to_string(),
            "run trellara run --local --verify for local pilot flows or relay/apply/verify for Kafka flows"
                .to_string(),
            "compare trellara status --view report across every flow before declaring fleet convergence"
                .to_string(),
            "rehearse quarantine replay-ready, checksum reseed, WAL-loss reseed, and schema handoff drills before the first production fleet".to_string(),
            "capture partner feature pull in pilot-package feature-pull-list.md after the first fleet review"
                .to_string(),
        ];

        Self {
            flow_count,
            source_count,
            dataset_count,
            table_count,
            target_configured_count,
            local_stream_count,
            kafka_stream_count,
            partitioned_flow_count,
            strict_chunked_flow_count,
            convergence_gate_count,
            blocked_convergence_gate_count,
            recovery_drill_count,
            lake_ready_flow_count,
            lake_publishable_with_gaps_flow_count,
            lake_blocked_flow_count,
            lake_fanin_verdict,
            topology_verdict,
            flows,
            warnings,
            proof_commands,
            next_commands,
        }
    }
}
