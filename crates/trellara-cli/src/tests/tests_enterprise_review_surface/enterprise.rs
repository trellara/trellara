use super::*;

#[test]
fn enterprise_evaluation_names_partitioned_visibility_contract() {
    let config = TrellaraConfig::from_yaml(&partitioned_yaml(), "partitioned.yml").expect("config");
    config.validate().expect("valid partitioned config");

    let summary = EnterpriseEvaluationSummary::from_config(&config, Path::new("partitioned.yml"));

    assert_eq!(summary.verdict, "ready_for_enterprise_pilot_evidence");
    assert_eq!(summary.recommended_mode, "partitioned_scale_mode");
    assert_eq!(
        summary.public_modes,
        vec![
            "strict_transaction_order".to_string(),
            "partitioned_scale_mode".to_string()
        ]
    );
    assert!(summary
        .mode_contract
        .contains("barrier-aware consumers wait for the manifest"));
    assert!(summary
        .transaction_boundary_contract
        .contains("partition-local analytics intentionally trade global ordering"));
    assert!(summary
        .proof_commands
        .contains(&"trellara partition-watermarks --config partitioned.yml".to_string()));
    assert!(summary.readiness_gates.iter().any(|gate| {
        gate.area == "source_safety"
            && gate.artifact == "source-safety.txt"
            && gate.command.contains("trellara check")
            && gate.pass_condition.contains("no critical")
    }));
    assert!(summary.readiness_gates.iter().any(|gate| {
        gate.area == "ddl_governance"
            && gate.artifact == "schema-ddl-plan.json"
            && gate.pass_condition.contains("active policy modes")
    }));
    assert!(summary.readiness_gates.iter().any(|gate| {
        gate.area == "partition_scale"
            && gate.artifact == "partition-watermarks.json"
            && gate.pass_condition.contains("no lagging")
            && gate.pass_condition.contains("global visibility")
    }));
    assert!(summary.readiness_gates.iter().any(|gate| {
        gate.area == "lake_fanin"
            && gate.artifact == "lake-verify.json"
            && gate.pass_condition.contains("manifest digest recomputes")
            && gate.pass_condition.contains("source_counts_match=true")
            && gate.pass_condition.contains("checksum_rollup_match=true")
    }));
    assert!(summary
        .differentiators
        .iter()
        .any(|item| item.contains("not a vague performance toggle")));
}

#[test]
fn enterprise_evaluation_text_renders_readiness_gate_matrix() {
    let config = TrellaraConfig::from_yaml(&partitioned_yaml(), "partitioned.yml").expect("config");
    let summary = EnterpriseEvaluationSummary::from_config(&config, Path::new("partitioned.yml"));

    let output = render_enterprise_evaluation_text(&summary);

    assert!(output.contains("readiness_gates:"));
    assert!(output.contains("- source_safety artifact=source-safety.txt"));
    assert!(output.contains("- ddl_governance artifact=schema-ddl-plan.json"));
    assert!(output
        .contains("- recovery_posture artifact=diagnostics.txt + local-stream-inspection.json"));
    assert!(output.contains("stream recovery.recovery_ready"));
    assert!(output.contains("- partition_scale artifact=partition-watermarks.json"));
    assert!(output.contains("- lake_fanin artifact=lake-verify.json"));
    assert!(output.contains("pass_condition=lake verification status is match"));
    assert!(output.contains("source_counts_match=true"));
    assert!(output.contains("checksum_rollup_match=true"));
}

#[test]
fn enterprise_evaluation_blocks_without_target_convergence() {
    let yaml_without_target = local_stream_yaml()
            .replace("\ntarget:\n  database_url: postgresql://trellara:trellara@localhost:55433/trellara_target\n", "\n");
    let config = TrellaraConfig::from_yaml(&yaml_without_target, "local.yml").expect("config");
    config.validate().expect("valid source-only config");

    let summary = EnterpriseEvaluationSummary::from_config(&config, Path::new("local.yml"));

    assert_eq!(summary.verdict, "blocked_before_enterprise_pilot");
    assert!(summary
        .buyer_summary
        .contains("target convergence is blocked"));
    assert!(summary
        .blockers
        .iter()
        .any(|blocker| blocker.contains("target.database_url is required")));
    assert!(summary.readiness_gates.iter().any(|gate| {
        gate.area == "recovery_posture"
            && gate.artifact == "diagnostics.txt + local-stream-inspection.json"
            && gate
                .command
                .contains("status --config local.yml --view diagnostics")
            && gate
                .command
                .contains("stream inspect-local --config local.yml")
            && gate.pass_condition.contains("recovery.recovery_ready")
            && gate.pass_condition.contains("torn-tail")
    }));
}
