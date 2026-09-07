use super::*;

#[test]
fn explain_reports_local_stream_path() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(config
        .explain()
        .contains("stream=local path=/tmp/trellara-local-stream"));
}

#[test]
fn strict_replay_redelivery_topics_use_configured_topic() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    assert_eq!(
        config
            .replay_redelivery_topics()
            .expect("redelivery topics"),
        vec!["trellara.local-source.retail-sales.strict".to_string()]
    );
}

#[test]
fn partitioned_replay_redelivery_topics_include_manifest_and_partitions() {
    let yaml = STRICT_YAML
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert_eq!(
        config
            .replay_redelivery_topics()
            .expect("redelivery topics"),
        vec![
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
            "trellara.local-source.retail-sales.partition.0".to_string(),
            "trellara.local-source.retail-sales.partition.1".to_string(),
            "trellara.local-source.retail-sales.partition.2".to_string(),
            "trellara.local-source.retail-sales.partition.3".to_string(),
        ]
    );
}
