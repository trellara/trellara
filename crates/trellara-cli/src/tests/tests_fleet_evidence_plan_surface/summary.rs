use super::*;

#[test]
fn fleet_evidence_plan_collects_live_evidence_gates() {
    let fixture = FleetEvidenceFixture::new("trellara-fleet-evidence-plan");
    let local_path = fixture.write_config("local.yml", local_stream_yaml());
    let partitioned_path = fixture.write_config("partitioned.yml", partitioned_yaml());

    let summary = FleetEvidencePlanSummary::from_args(&FleetEvidencePlanArgs {
        config: vec![local_path.clone(), partitioned_path.clone()],
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet evidence plan");

    assert_eq!(summary.verdict, "ready_to_collect_live_evidence");
    assert_eq!(summary.flow_count, 2);
    assert_eq!(summary.blocked_gate_count, 0);
    assert!(summary.needs_live_evidence_gate_count > 0);
    assert_eq!(
        summary.required_live_evidence_artifact_count,
        summary.needs_live_evidence_gate_count
    );
    assert!(summary
        .command_sequence
        .iter()
        .any(|command| { command.contains("trellara check --config") }));
    assert!(summary
        .command_sequence
        .iter()
        .any(|command| { command.contains("trellara partition-watermarks --config") }));
    assert!(summary
        .command_sequence
        .iter()
        .any(|command| { command.contains("trellara partition-rebalance-plan --config") }));
    assert!(summary
        .command_sequence
        .iter()
        .any(|command| { command.contains("trellara lake fanin verify --config") }));
    assert!(summary
        .command_sequence
        .iter()
        .any(|command| { command.contains("trellara pilot evidence-template --config") }));
    assert!(summary
        .command_sequence
        .iter()
        .any(|command| { command.contains("trellara pilot evidence-check --config") }));
    let partitioned_flow = summary
        .flows
        .iter()
        .find(|flow| flow.config == partitioned_path.display().to_string())
        .expect("partitioned evidence flow");
    assert!(partitioned_flow
        .live_evidence_gates
        .iter()
        .any(|gate| gate.code == "partition_watermarks"));
    assert!(partitioned_flow.live_evidence_gates.iter().any(|gate| {
        gate.code == "partition_rebalance_plan"
            && gate.artifact == "partition-rebalance-plan.json"
            && gate
                .success_markers
                .contains(&"runtime movement disabled".to_string())
            && gate
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("reviewed cutover evidence"))
    }));
    assert_eq!(
        partitioned_flow.required_live_evidence_artifact_count,
        partitioned_flow.needs_live_evidence_gate_count
    );
    assert!(partitioned_flow.live_evidence_gates.iter().any(|gate| {
        gate.code == "lake_writer_plan"
            && gate.artifact == "lake-writer-plan.json"
            && gate
                .success_markers
                .contains(&"DDL boundary metadata".to_string())
            && gate
                .success_markers
                .contains(&"partition source evidence".to_string())
            && gate
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("partition source_id"))
    }));
    assert!(partitioned_flow.live_evidence_gates.iter().any(|gate| {
        gate.code == "lake_spark_consumption"
            && gate.artifact == "lake-completeness.json"
            && gate
                .success_markers
                .contains(&"spark_consumption_allowed true".to_string())
            && gate
                .success_markers
                .contains(&"spark consumption contract".to_string())
            && gate
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("spark_consumption_contract"))
            && gate
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("released Spark consumption gate"))
    }));
    assert!(summary
        .review_rule
        .contains("collect every needs_live_evidence command"));
}

#[test]
fn fleet_evidence_plan_blocks_when_flow_config_is_blocked() {
    let fixture = FleetEvidenceFixture::new("trellara-fleet-evidence-plan-blocked");
    let yaml = local_stream_yaml().replace(
        "\ntarget:\n  database_url: postgresql://trellara:trellara@localhost:55433/trellara_target\n",
        "\n",
    );
    let config_path = fixture.write_config("source-only.yml", yaml);

    let summary = FleetEvidencePlanSummary::from_args(&FleetEvidencePlanArgs {
        config: vec![config_path],
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet evidence plan");

    assert_eq!(summary.verdict, "blocked_until_config_fixed");
    assert!(summary.blocked_gate_count > 0);
    assert!(summary.flows[0]
        .blocked_gates
        .iter()
        .any(|gate| gate.code == "verified_apply"));
}
