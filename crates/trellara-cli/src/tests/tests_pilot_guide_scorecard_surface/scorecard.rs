use super::*;

#[test]
fn pilot_scorecard_marks_local_config_ready_for_live_evidence() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse");
    let scorecard = PilotScorecardSummary::from_config(&config, Path::new("trellara.yml"));

    assert_eq!(scorecard.verdict, "ready_for_live_pilot");
    assert_eq!(scorecard.blocked_gate_count, 0);
    assert!(scorecard.score >= 90);
    assert!(scorecard.gates.iter().any(|gate| {
        gate.code == "brokerless_evaluation"
            && gate.status == PilotScorecardStatus::ConfigurationReady
            && gate.evidence.contains("embedded durable stream")
    }));
    assert!(scorecard.gates.iter().any(|gate| {
        gate.code == "bounded_memory_capture"
            && gate.status == PilotScorecardStatus::ConfigurationReady
            && gate
                .evidence
                .contains("pgoutput streamed transaction changes spill")
            && gate.acceptance.contains("bounded_memory_contract")
            && gate.acceptance.contains("visibility_contract")
    }));
    assert!(scorecard.next_commands.contains(
            &"trellara run --local --verify --format text --config trellara.yml --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"
                .to_string()
        ));
}

#[test]
fn pilot_scorecard_blocks_verified_pilot_without_target() {
    let yaml = local_stream_yaml().replace(
        "target:\n  database_url: postgresql://trellara:trellara@localhost:55433/trellara_target\n",
        "",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let scorecard = PilotScorecardSummary::from_config(&config, Path::new("trellara.yml"));

    assert_eq!(scorecard.verdict, "blocked_before_verified_pilot");
    assert_eq!(scorecard.blocked_gate_count, 1);
    let verified_apply = scorecard
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");
    assert_eq!(verified_apply.status, PilotScorecardStatus::Blocked);
    assert!(verified_apply
        .evidence
        .contains("target.database_url is missing"));
}

#[test]
fn pilot_scorecard_surfaces_partitioned_watermark_gate() {
    let config = TrellaraConfig::from_yaml(&local_partitioned_yaml(), "test").expect("parse");
    let scorecard = PilotScorecardSummary::from_config(
        &config,
        Path::new("examples/retail-fleet/partitioned.yml"),
    );

    assert!(scorecard.gates.iter().any(|gate| {
        gate.code == "partition_watermarks"
            && gate.status == PilotScorecardStatus::NeedsLiveEvidence
            && gate.proof_command
                == "trellara partition-watermarks --config examples/retail-fleet/partitioned.yml"
    }));
    assert!(scorecard.gates.iter().any(|gate| {
        gate.code == "partition_rebalance_plan"
            && gate.status == PilotScorecardStatus::NeedsLiveEvidence
            && gate
                .proof_command
                .contains("trellara partition-rebalance-plan --config")
            && gate.acceptance.contains("runtime_movement_allowed=false")
    }));
    assert!(scorecard.next_commands.contains(
        &"trellara partition-watermarks --config examples/retail-fleet/partitioned.yml".to_string()
    ));
    assert!(scorecard
        .next_commands
        .iter()
        .any(|command| command.contains(
            "trellara partition-rebalance-plan --config examples/retail-fleet/partitioned.yml"
        )));
    assert!(scorecard.next_commands.contains(
        &"trellara stream reconstruct-local --config examples/retail-fleet/partitioned.yml --transaction-id <tx> --commit-lsn <lsn>"
            .to_string()
    ));
}

#[test]
fn pilot_scorecard_text_renders_acceptance_gates() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse");
    let scorecard = PilotScorecardSummary::from_config(&config, Path::new("trellara.yml"));
    let output = render_pilot_scorecard_text(&scorecard);

    assert!(output.contains("Trellara pilot scorecard"));
    assert!(output.contains("verdict: ready_for_live_pilot"));
    assert!(output.contains("[needs_live_evidence] transaction_boundary"));
    assert!(output.contains("transaction boundary is proven from live checkpoints"));
    assert!(output.contains("[needs_live_evidence] source_safety"));
    assert!(
        output.contains("trellara status --config trellara.yml --view diagnostics --format text")
    );
}
