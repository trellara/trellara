use super::*;

#[test]
fn fleet_report_blocks_convergence_when_target_is_missing() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-report-missing-target-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet report missing target temp dir");
    let config_path = root.join("source-only.yml");
    let yaml = local_stream_yaml().replace(
        "target:\n  database_url: postgresql://trellara:trellara@localhost:55433/trellara_target\n",
        "",
    );
    fs::write(&config_path, yaml).expect("write source-only config");

    let summary = FleetReportSummary::from_args(&FleetReportArgs {
        config: vec![config_path],
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet report");

    assert_eq!(summary.topology_verdict, "needs_targets");
    assert_eq!(summary.target_configured_count, 0);
    assert_eq!(summary.blocked_convergence_gate_count, 1);
    assert_eq!(summary.recovery_drill_count, 7);
    assert_eq!(summary.lake_fanin_verdict, "publishable_with_gaps");
    let target_gate = summary.flows[0]
        .convergence_gates
        .iter()
        .find(|gate| gate.code == "target_convergence")
        .expect("target gate");
    assert_eq!(
        target_gate.status,
        FleetConvergenceGateStatus::BlockedByConfig
    );
    assert!(target_gate
        .evidence
        .contains("target.database_url is missing"));
    assert!(summary
        .warnings
        .iter()
        .any(|warning| { warning.contains("verification and convergence evidence are blocked") }));

    fs::remove_dir_all(root).expect("remove fleet report missing target temp dir");
}
