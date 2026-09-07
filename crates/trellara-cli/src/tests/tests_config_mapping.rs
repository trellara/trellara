use super::*;

#[test]
#[cfg(feature = "kafka")]
fn maps_yaml_to_kafka_publisher_config() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let publisher = config
        .to_kafka_publisher_config()
        .expect("publisher config");

    assert_eq!(publisher.bootstrap_servers, "localhost:9092");
    assert_eq!(
        publisher.client_id,
        "trellara-relay-local-source-retail-sales"
    );
    assert!(publisher.enable_idempotence);
}

#[test]
fn strict_config_maps_to_strict_relay_mode() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    assert_eq!(
        config.to_relay_mode().expect("relay mode"),
        RelayMode::Strict
    );
}

#[test]
#[cfg(feature = "kafka")]
fn strict_chunking_config_maps_to_chunked_relay_and_barrier_topics() {
    let yaml = STRICT_YAML.replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 1000",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    config.validate().expect("valid strict chunking");
    assert_eq!(
        config.to_relay_mode().expect("relay mode"),
        RelayMode::StrictChunked(StrictChunkPlanConfig {
            max_changes_per_chunk: 1000,
        })
    );
    assert_eq!(config.status_mode(), "strict_chunked_transaction_order");
    assert!(config.uses_barrier_apply());
    assert_eq!(
        config
            .to_kafka_consumer_config()
            .expect("consumer config")
            .topics,
        vec![
            "trellara.local-source.retail-sales.strict".to_string(),
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
        ]
    );
    assert_eq!(
        config.replay_redelivery_topics().expect("replay topics"),
        vec![
            "trellara.local-source.retail-sales.strict".to_string(),
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
        ]
    );
    assert!(config
        .explain()
        .contains("chunked at 1000 changes per chunk"));
}

#[test]
fn local_strict_chunking_subscribes_to_strict_and_manifest_topics() {
    let yaml = local_stream_yaml().replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 2",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert_eq!(
        config.local_stream_topics().expect("local topics"),
        vec![
            "trellara.local-source.retail-sales.strict".to_string(),
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
        ]
    );
    assert_eq!(
        config
            .to_local_consumer_config()
            .expect("local consumer config")
            .topics,
        vec![
            "trellara.local-source.retail-sales.strict".to_string(),
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
        ]
    );
}

#[test]
fn strict_chunking_requires_positive_chunk_size() {
    let yaml = STRICT_YAML.replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 0",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "dataset.strict_chunking.max_changes_per_chunk must be greater than zero"
    ));
}

#[test]
fn strict_chunking_is_rejected_for_partitioned_mode() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 16\n    key_column: store_id\n  strict_chunking:\n    max_changes_per_chunk: 1000",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "dataset.strict_chunking is only supported for strict_transaction_order"
    ));
}
