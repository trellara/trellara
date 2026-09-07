use super::*;

#[test]
fn local_replay_redelivery_topics_use_generated_strict_layout() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert_eq!(
        config
            .replay_redelivery_topics()
            .expect("redelivery topics"),
        vec!["trellara.local-source.retail-sales.strict".to_string()]
    );
}

#[test]
fn local_partitioned_mode_subscribes_to_manifest_and_partition_topics() {
    let yaml = local_stream_yaml()
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 2\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let consumer = config.to_local_consumer_config().expect("consumer config");

    assert_eq!(
        consumer.topics,
        vec![
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
            "trellara.local-source.retail-sales.partition.0".to_string(),
            "trellara.local-source.retail-sales.partition.1".to_string(),
        ]
    );
}
