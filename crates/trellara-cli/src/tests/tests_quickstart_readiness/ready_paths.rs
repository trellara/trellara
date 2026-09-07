use super::*;

#[test]
fn quickstart_readiness_accepts_local_pgoutput_config() {
    let fixture = quickstart_config("trellara-quickstart-ready", local_stream_yaml());

    let summary = readiness_summary(fixture.config_path.clone());

    assert!(summary.ready);
    assert_eq!(
        summary.estimated_minutes,
        Some(QUICKSTART_ESTIMATED_MINUTES)
    );
    assert_eq!(summary.time_budget_minutes, QUICKSTART_TIME_BUDGET_MINUTES);
    assert_eq!(
        summary.evidence_bundle.as_deref(),
        Some("target/trellara-quickstart-evidence")
    );
    let expected_recovery_command = format!(
        "trellara status --config {} --view diagnostics --format text",
        fixture.config_path.display()
    );
    assert_eq!(
        summary.recovery_command.as_deref(),
        Some(expected_recovery_command.as_str())
    );
    assert_eq!(summary.check_count, summary.passed_check_count);
    let boundary = summary
        .checks
        .iter()
        .find(|check| check.code == "transaction_boundary")
        .expect("transaction boundary check");
    assert!(boundary.message.contains("strict single-envelope boundary"));
    let spill = summary
        .checks
        .iter()
        .find(|check| check.code == "capture_spill_boundary")
        .expect("capture spill check");
    assert!(spill
        .message
        .contains("pgoutput streamed transaction changes spill after 1024 changes"));
    assert!(spill.message.contains("OS temp directory"));
    let source_safety_command = format!(
        "trellara check --config {} --format text",
        fixture.config_path.display()
    );
    let preflight_command = format!(
        "trellara preflight --config {}",
        fixture.config_path.display()
    );
    assert!(summary.next_commands.contains(&source_safety_command));
    assert!(summary.next_commands.contains(&preflight_command));
    let source_safety_position = summary
        .next_commands
        .iter()
        .position(|command| command == &source_safety_command)
        .expect("source-safety next command");
    let preflight_position = summary
        .next_commands
        .iter()
        .position(|command| command == &preflight_command)
        .expect("preflight next command");
    assert!(source_safety_position < preflight_position);
    assert!(summary.next_commands.contains(&format!(
        "trellara run --local --verify --format text --config {} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100",
        fixture.config_path.display()
    )));
    assert!(summary.next_commands.contains(&format!(
        "trellara status --config {} --view report --format text",
        fixture.config_path.display()
    )));
    assert!(summary.next_commands.contains(&format!(
        "trellara pilot-package --config {} --output target/trellara-quickstart-evidence",
        fixture.config_path.display()
    )));
    assert!(!summary
        .next_commands
        .iter()
        .any(|command| command.contains("contract-test")));
    assert!(!summary
        .next_commands
        .iter()
        .any(|command| command.contains("pilot-guide")));
}

#[test]
fn quickstart_readiness_reports_strict_chunk_manifest_boundary() {
    let yaml = local_stream_yaml().replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  strict_chunking:\n    max_changes_per_chunk: 1000\n  tables:\n    - schema: public\n      name: sales",
    );
    let fixture = quickstart_config("trellara-quickstart-chunk-ready", yaml);

    let summary = readiness_summary(fixture.config_path.clone());

    assert!(summary.ready);
    assert_eq!(summary.check_count, 9);
    let boundary = summary
        .checks
        .iter()
        .find(|check| check.code == "transaction_boundary")
        .expect("transaction boundary check");
    assert!(boundary.message.contains(
        "strict chunk manifest and commit marker boundary enabled at 1000 changes per chunk"
    ));
    assert!(summary
        .checks
        .iter()
        .any(|check| check.code == "capture_spill_boundary"));
}
