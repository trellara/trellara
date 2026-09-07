use super::*;

#[test]
fn quickstart_readiness_reports_missing_config_without_error() {
    let summary = readiness_summary(PathBuf::from("/tmp/trellara-missing-quickstart.yml"));

    assert!(!summary.ready);
    assert_eq!(summary.estimated_minutes, None);
    assert_eq!(summary.time_budget_minutes, QUICKSTART_TIME_BUDGET_MINUTES);
    assert_eq!(summary.check_count, 1);
    assert_eq!(summary.checks[0].code, "config_readable");
    assert!(summary.checks[0]
        .fix
        .as_deref()
        .unwrap()
        .contains("trellara init"));
}

#[test]
fn quickstart_readiness_rejects_kafka_demo_config() {
    let fixture = quickstart_config("trellara-quickstart-kafka", STRICT_YAML);

    let summary = readiness_summary(fixture.config_path.clone());

    assert!(!summary.ready);
    let local_stream = summary
        .checks
        .iter()
        .find(|check| check.code == "local_stream")
        .expect("local stream check");
    assert!(!local_stream.passed);
    assert!(local_stream.message.contains("not local"));
}
