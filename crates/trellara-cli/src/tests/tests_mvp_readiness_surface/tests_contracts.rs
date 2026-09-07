use super::*;

#[test]
fn mvp_check_rejects_non_streaming_pgoutput_capture_contract() {
    let root = std::env::temp_dir().join(format!("trellara-mvp-pgoutput-{}", std::process::id()));
    let config_path = root.join("trellara.yml");
    let yaml = local_stream_yaml().replace(
        "  slot: trellara_retail_slot",
        "  slot: trellara_retail_slot\n  pgoutput:\n    protocol_version: 1\n    streaming: false",
    );
    fs::create_dir_all(&root).expect("create mvp temp dir");
    fs::write(&config_path, yaml).expect("write config");
    let config = TrellaraConfig::from_path(&config_path).expect("parse config");

    let summary = MvpReadinessSummary::from_config(&config, &config_path).expect("mvp readiness");

    assert!(!summary.ready);
    let pgoutput = criterion(&summary, "pgoutput_capture_path");
    assert!(!pgoutput.passed);
    assert_contains_all(
        &pgoutput.evidence,
        &["pgoutput.protocol_version=1", "pgoutput.streaming=false"],
    );
    assert!(summary.next_commands.iter().any(|command| {
        command.contains(&format!(
            "trellara check --config {} --format text",
            config_path.display()
        )) && command.contains("slot_plugin_guard")
    }));
    assert_eq!(summary.priority_next_commands.len(), 1);
    assert_contains_all(
        &summary.priority_next_commands[0],
        &[
            "trellara check --config",
            "slot_plugin_guard",
            &config_path.display().to_string(),
        ],
    );

    fs::remove_dir_all(root).expect("remove mvp temp dir");
}

#[test]
fn mvp_check_verifies_correctness_report_workflow_contract() {
    let repository_root = workspace_path(".");

    assert!(correctness_report_workflow_publishes_tested_report(
        &repository_root
    ));
}
