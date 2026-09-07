use super::*;

#[test]
fn flow_stream_summary_describes_local_segment_log() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse");

    let summary = FlowStreamSummary::from_config(&config.stream);

    assert_eq!(summary.kind, "local");
    assert!(summary.bootstrap_servers.ends_with("trellara-local-stream"));
    assert_eq!(summary.primary_topic, "local durable segment log");
}

#[test]
fn flow_stream_summary_describes_kafka_topic() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    let summary = FlowStreamSummary::from_config(&config.stream);

    assert_eq!(summary.kind, "kafka");
    assert_eq!(summary.bootstrap_servers, "localhost:9092");
    assert_eq!(
        summary.primary_topic,
        "trellara.local-source.retail-sales.strict"
    );
}
