use super::*;

#[test]
fn flow_create_summary_reports_local_stream() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = FlowCreateSummary::from_config(&config, Path::new("local.yml"));

    assert_eq!(summary.stream.kind, "local");
    assert_eq!(
        summary.stream.bootstrap_servers,
        "/tmp/trellara-local-stream"
    );
    assert_eq!(summary.stream.primary_topic, "local durable segment log");
}

#[test]
fn flow_create_summary_recommends_local_stream_inspection() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = FlowCreateSummary::from_config(&config, Path::new("local.yml"));

    assert!(summary
        .next_commands
        .contains(&"trellara stream inspect-local --config local.yml".to_string()));
}
