use super::*;

#[test]
fn fleet_report_text_surfaces_verdict_and_proof_commands() {
    let strict_chunked_local_yaml = local_stream_yaml().replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 1000",
    );
    let config = TrellaraConfig::from_yaml(&strict_chunked_local_yaml, "local").expect("parse");
    let flow = FleetFlowSummary::from_config(&config, Path::new("local.yml")).expect("flow");
    let convergence_gate_count = flow.convergence_gates.len();
    let recovery_drill_count = flow.recovery_drills.len();
    let summary = FleetReportSummary {
        flow_count: 1,
        source_count: 1,
        dataset_count: 1,
        table_count: 1,
        target_configured_count: 1,
        local_stream_count: 1,
        kafka_stream_count: 0,
        partitioned_flow_count: 0,
        strict_chunked_flow_count: 1,
        convergence_gate_count,
        blocked_convergence_gate_count: 0,
        recovery_drill_count,
        lake_ready_flow_count: 0,
        lake_publishable_with_gaps_flow_count: 1,
        lake_blocked_flow_count: 0,
        lake_fanin_verdict: "publishable_with_gaps".to_string(),
        topology_verdict: "ready_for_design_partner_review".to_string(),
        flows: vec![flow],
        warnings: Vec::new(),
        proof_commands: vec![
            "trellara check --config local.yml".to_string(),
            "trellara status --config local.yml --view report --format text".to_string(),
            "trellara lake fanin verify --config local.yml --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json>".to_string(),
            "trellara lake spark-template dashboard --config local.yml --table <schema.table> --epoch-id <epoch-id>".to_string(),
        ],
        next_commands: vec!["compare flow reports before declaring convergence".to_string()],
    };

    let output = render_fleet_report_text(&summary);

    assert!(output.contains("Trellara fleet report"));
    assert!(output.contains("verdict: ready_for_design_partner_review"));
    assert!(output.contains("streams: local=1 kafka=0"));
    assert!(output.contains("convergence_gates: 6 total, 0 blocked_by_config"));
    assert!(output.contains("recovery_drills: 7"));
    assert!(output.contains(
        "lake_fanin: verdict=publishable_with_gaps ready=0 publishable_with_gaps=1 blocked=0"
    ));
    assert!(
        output.contains("lake_fanin: status=publishable_with_gaps mode=strict_chunked_epoch_fanin")
    );
    assert!(output.contains("iceberg_checkpoint_receipts: epoch metadata is publishable only after every planned Iceberg table append has a matching checkpoint receipt"));
    assert!(output.contains("watermarks: strict chunk manifests roll up"));
    assert!(output.contains("[needs_live_evidence] target_convergence"));
    assert!(output.contains("target_quarantine_replay"));
    assert!(output.contains("schema_handoff_refresh"));
    assert!(output.contains("lake_offline_source_gap_acceptance"));
    assert!(output.contains("lake_late_source_recompute"));
    assert!(output.contains("lake_conflicting_duplicate_quarantine"));
    assert!(output.contains("local_stream_durability"));
    assert!(output.contains("local-source:retail-sales"));
    assert!(output.contains("strict chunked"));
    assert!(output.contains("trellara check --config local.yml"));
    assert!(output.contains("trellara lake fanin verify --config local.yml"));
    assert!(output.contains("trellara lake spark-template dashboard --config local.yml"));
    assert!(output.contains("compare flow reports before declaring convergence"));
}
