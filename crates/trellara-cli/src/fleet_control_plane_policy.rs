use crate::{
    FleetControlPlaneCapability, FleetControlPlaneCapabilityStatus, FleetReportSummary,
    FleetScorecardSummary,
};

pub(crate) fn fleet_control_plane_evidence_gaps(
    report: &FleetReportSummary,
    scorecard: &FleetScorecardSummary,
) -> Vec<String> {
    let mut gaps = Vec::new();
    if report.flow_count < 2 {
        gaps.push(
            "need at least two flow configs before hosted fleet topology is a real pull"
                .to_string(),
        );
    }
    if report.target_configured_count < report.flow_count {
        gaps.push(
            "every flow needs target.database_url before hosted convergence evidence is meaningful"
                .to_string(),
        );
    }
    if scorecard.blocked_flow_count > 0 || scorecard.blocked_gate_count > 0 {
        gaps.push(
            "blocked pilot gates must be resolved before Cloud can be scoped honestly".to_string(),
        );
    }
    if scorecard.needs_live_evidence_gate_count > 0 {
        gaps.push("live source-safety, contract, verification, and recovery-drill evidence is still required"
            .to_string());
    }
    if report.recovery_drill_count < report.flow_count.saturating_mul(4) {
        gaps.push(
            "fleet recovery drills are not broad enough to justify hosted orchestration"
                .to_string(),
        );
    }
    gaps
}

pub(crate) fn fleet_control_plane_capabilities(
    report: &FleetReportSummary,
    scorecard: &FleetScorecardSummary,
    evidence_gaps: &[String],
) -> Vec<FleetControlPlaneCapability> {
    let has_many_flows = report.flow_count >= 2;
    let all_targets_configured = report.target_configured_count == report.flow_count;
    let no_identity_collisions = !report
        .warnings
        .iter()
        .any(|warning| warning.contains("control-plane identity would collide"));
    let no_blockers = scorecard.blocked_flow_count == 0 && scorecard.blocked_gate_count == 0;
    let proof_is_reasonable =
        evidence_gaps.is_empty() || (has_many_flows && all_targets_configured);

    vec![
        FleetControlPlaneCapability {
            code: "read_only_fleet_topology".to_string(),
            status: if has_many_flows && no_identity_collisions {
                FleetControlPlaneCapabilityStatus::PulledNow
            } else {
                FleetControlPlaneCapabilityStatus::ValidateWithPartners
            },
            rationale: "multiple flow configs create identity, ownership, and review-sequence pressure that local files start to hide"
                .to_string(),
            evidence: vec![
                format!("flows={}", report.flow_count),
                format!("sources={}", report.source_count),
                format!("datasets={}", report.dataset_count),
                format!("topology_verdict={}", report.topology_verdict),
            ],
            build_when: "two or more partner-owned flows need one read-only topology and review sequence"
                .to_string(),
        },
        FleetControlPlaneCapability {
            code: "evidence_package_registry".to_string(),
            status: if proof_is_reasonable && no_blockers {
                FleetControlPlaneCapabilityStatus::PulledNow
            } else {
                FleetControlPlaneCapabilityStatus::ValidateWithPartners
            },
            rationale: "buyers need immutable links to source-safety, pilot package, correctness, diagnostics, and fleet proof artifacts"
                .to_string(),
            evidence: vec![
                format!("proof_commands={}", report.proof_commands.len()),
                format!("scorecard_verdict={}", scorecard.verdict),
                format!("needs_live_evidence_gates={}", scorecard.needs_live_evidence_gate_count),
            ],
            build_when:
                "design partners repeatedly ask to share proof bundles outside the local repo"
                    .to_string(),
        },
        FleetControlPlaneCapability {
            code: "policy_exception_workflow".to_string(),
            status: FleetControlPlaneCapabilityStatus::ValidateWithPartners,
            rationale: "source-safety decisions may require approvals, exceptions, and compliance evidence, but the current MVP should collect that pull first"
                .to_string(),
            evidence: report.warnings.clone(),
            build_when:
                "source-safety blockers require named approvers or audit trails during partner reviews"
                    .to_string(),
        },
        FleetControlPlaneCapability {
            code: "deployment_orchestration".to_string(),
            status: FleetControlPlaneCapabilityStatus::Defer,
            rationale: "rollout automation is expensive and should wait until recurring multi-node deployment burden is measured"
                .to_string(),
            evidence: vec![
                format!("local_streams={}", report.local_stream_count),
                format!("kafka_streams={}", report.kafka_stream_count),
            ],
            build_when:
                "operators repeat the same verified rollout across many Postgres nodes and local scripts are the bottleneck"
                    .to_string(),
        },
        FleetControlPlaneCapability {
            code: "hosted_stream_management".to_string(),
            status: FleetControlPlaneCapabilityStatus::Defer,
            rationale: "managed stream ownership should not precede proof that local transport or customer Kafka is insufficient"
                .to_string(),
            evidence: vec![
                format!("local_streams={}", report.local_stream_count),
                format!("kafka_streams={}", report.kafka_stream_count),
            ],
            build_when:
                "partners explicitly reject both brokerless local streams and existing Kafka ownership for fleet operations"
                    .to_string(),
        },
    ]
}
