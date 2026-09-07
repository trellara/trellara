use super::*;

mod fixtures;
use fixtures::*;

#[test]
fn fleet_scorecard_aggregates_flow_readiness() {
    let fixture = mixed_fleet_scorecard_fixture();

    let summary = FleetScorecardSummary::from_args(&FleetScorecardArgs {
        config: fixture.configs(),
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet scorecard");

    assert_eq!(summary.flow_count, 2);
    assert_eq!(summary.verdict, "review_required_before_fleet_pilot");
    assert_eq!(summary.topology_verdict, "review_required");
    assert_eq!(summary.ready_flow_count, 1);
    assert_eq!(summary.review_required_flow_count, 1);
    assert_eq!(summary.blocked_flow_count, 0);
    assert_eq!(summary.local_stream_count, 1);
    assert_eq!(summary.kafka_stream_count, 1);
    assert_eq!(summary.target_configured_count, 2);
    assert_eq!(summary.blocked_gate_count, 0);
    assert_eq!(summary.blocked_convergence_gate_count, 0);
    assert_eq!(summary.convergence_gate_count, 12);
    assert_eq!(summary.recovery_drill_count, 16);
    assert_eq!(summary.lake_ready_flow_count, 0);
    assert_eq!(summary.lake_publishable_with_gaps_flow_count, 2);
    assert_eq!(summary.lake_blocked_flow_count, 0);
    assert_eq!(summary.lake_fanin_verdict, "publishable_with_gaps");
    assert!(summary.warnings.iter().any(|warning| {
        warning.contains("duplicate flow_id local-source:retail-sales appears 2 times")
    }));
    assert!(summary
        .next_commands
        .first()
        .expect("fleet report command")
        .contains("trellara fleet report --config"));
    assert!(summary
        .review_sequence
        .iter()
        .any(|step| step.contains("prove one brokerless local flow")));
    let partitioned_flow = summary
        .flows
        .iter()
        .find(|flow| flow.config == fixture.partitioned_display())
        .expect("partitioned flow");
    assert_eq!(partitioned_flow.verdict, "ready_for_live_pilot");
    assert_eq!(partitioned_flow.mode, "partitioned_scale_mode");
    assert_eq!(partitioned_flow.stream_kind, "kafka");
    assert_eq!(partitioned_flow.lake_fanin_status, "publishable_with_gaps");
    assert_eq!(
        partitioned_flow.lake_fanin_mode,
        "partitioned_scale_epoch_fanin"
    );
    assert!(partitioned_flow
        .risks
        .iter()
        .any(|risk| risk.contains("partitioned scale requires manifest")));

    fixture.remove();
}

#[test]
fn fleet_scorecard_blocks_when_target_is_missing() {
    let fixture = source_only_scorecard_fixture();

    let summary = FleetScorecardSummary::from_args(&FleetScorecardArgs {
        config: fixture.configs(),
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet scorecard");

    assert_eq!(summary.verdict, "blocked_before_fleet_pilot");
    assert_eq!(summary.blocked_flow_count, 1);
    assert_eq!(summary.ready_flow_count, 0);
    assert_eq!(summary.target_configured_count, 0);
    assert!(summary.blocked_gate_count > 0);
    assert!(summary.blocked_convergence_gate_count > 0);
    assert!(summary
        .blockers
        .iter()
        .any(|blocker| blocker.contains("blocked pilot gate")));
    assert!(summary
        .blockers
        .iter()
        .any(|blocker| blocker.contains("blocked by config")));

    fixture.remove();
}

#[test]
fn fleet_scorecard_text_surfaces_verdict_blockers_and_sequence() {
    let flow = FleetScorecardFlow {
        flow_id: "source-a:sales".to_string(),
        config: "sales.yml".to_string(),
        mode: "strict_chunked_transaction_order".to_string(),
        stream_kind: "local".to_string(),
        target_configured: true,
        verdict: "ready_for_live_pilot".to_string(),
        score: 100,
        gate_count: 8,
        needs_live_evidence_gate_count: 4,
        blocked_gate_count: 0,
        convergence_gate_count: 6,
        blocked_convergence_gate_count: 0,
        recovery_drill_count: 7,
        lake_fanin_status: "ready".to_string(),
        lake_fanin_mode: "strict_chunked_epoch_fanin".to_string(),
        risks: Vec::new(),
    };
    let summary = FleetScorecardSummary {
        flow_count: 1,
        verdict: "ready_for_design_partner_fleet_review".to_string(),
        score: 100,
        topology_verdict: "ready_for_design_partner_review".to_string(),
        ready_flow_count: 1,
        review_required_flow_count: 0,
        blocked_flow_count: 0,
        local_stream_count: 1,
        kafka_stream_count: 0,
        target_configured_count: 1,
        configuration_ready_gate_count: 4,
        needs_live_evidence_gate_count: 4,
        blocked_gate_count: 0,
        convergence_gate_count: 6,
        blocked_convergence_gate_count: 0,
        recovery_drill_count: 7,
        lake_ready_flow_count: 1,
        lake_publishable_with_gaps_flow_count: 0,
        lake_blocked_flow_count: 0,
        lake_fanin_verdict: "ready".to_string(),
        flows: vec![flow],
        blockers: Vec::new(),
        warnings: Vec::new(),
        review_sequence: vec![
            "run source-safety and contract-test for every flow".to_string(),
            "compare fleet report and fleet scorecard before design-partner review".to_string(),
        ],
        next_commands: vec!["trellara fleet report --config sales.yml --format text".to_string()],
    };

    let output = render_fleet_scorecard_text(&summary);

    assert!(output.contains("Trellara fleet scorecard"));
    assert!(output.contains("verdict: ready_for_design_partner_fleet_review"));
    assert!(output.contains("flow_status: 1 ready, 0 review_required, 0 blocked"));
    assert!(output
        .contains("analytical_fanin: verdict=ready ready=1 publishable_with_gaps=0 blocked=0"));
    assert!(output.contains("source-a:sales config=sales.yml"));
    assert!(output.contains("convergence_gates: 6 total, 0 blocked_by_config"));
    assert!(output.contains("analytical_fanin: status=ready mode=strict_chunked_epoch_fanin"));
    assert!(output.contains("review_sequence:"));
    assert!(output.contains("trellara fleet report --config sales.yml --format text"));
}
