use super::*;

#[test]
fn partitioned_config_maps_to_partitioned_relay_mode() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 16\n    key_column: store_id",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert_eq!(
        config.to_relay_mode().expect("relay mode"),
        RelayMode::Partitioned(PartitionPlanConfig {
            partition_count: 16,
            key_column: "store_id".to_string(),
            null_key_policy: ProtocolPartitionNullKeyPolicy::Quarantine,
            key_change_policy: ProtocolPartitionKeyChangePolicy::Quarantine,
        })
    );
    let partition = config.dataset.partition.expect("partition config");
    assert_eq!(
        partition.null_key_policy,
        PartitionNullKeyPolicy::Quarantine
    );
    assert_eq!(
        partition.key_change_policy,
        PartitionKeyChangePolicy::Quarantine
    );
}

#[test]
fn partitioned_config_parses_partition_key_policies() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 16\n    key_column: store_id\n    null_key_policy: route_to_dead_letter_partition\n    key_change_policy: emit_move",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    assert_eq!(
        config.to_relay_mode().expect("relay mode"),
        RelayMode::Partitioned(PartitionPlanConfig {
            partition_count: 16,
            key_column: "store_id".to_string(),
            null_key_policy: ProtocolPartitionNullKeyPolicy::RouteToDeadLetterPartition,
            key_change_policy: ProtocolPartitionKeyChangePolicy::EmitMove,
        })
    );
    let partition = config.dataset.partition.expect("partition config");

    assert_eq!(
        partition.null_key_policy,
        PartitionNullKeyPolicy::RouteToDeadLetterPartition
    );
    assert_eq!(
        partition.key_change_policy,
        PartitionKeyChangePolicy::EmitMove
    );
}

#[test]
#[cfg(feature = "kafka")]
fn explicit_consumer_group_overrides_default() {
    let yaml = STRICT_YAML.replace(
        "  topic: trellara.local-source.retail-sales.strict",
        "  topic: trellara.local-source.retail-sales.strict\n  consumer_group: custom-applier",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let consumer = config.to_kafka_consumer_config().expect("consumer config");

    assert_eq!(consumer.group_id, "custom-applier");
}

#[test]
#[cfg(feature = "kafka")]
fn partitioned_mode_subscribes_to_manifest_and_partition_topics() {
    let yaml = STRICT_YAML
            .replace("strict_transaction_order", "partitioned_scale_mode")
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 4\n    key_column: store_id",
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let consumer = config.to_kafka_consumer_config().expect("consumer config");

    assert_eq!(
        consumer.topics,
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

#[test]
fn partitioned_mode_requires_partition_settings() {
    let config = TrellaraConfig::from_yaml(
        &STRICT_YAML.replace("strict_transaction_order", "partitioned_scale_mode"),
        "test",
    )
    .expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "dataset.partition is required for partitioned_scale_mode"
    ));
}

#[test]
fn partitioned_mode_explains_transaction_barrier() {
    let yaml = STRICT_YAML.replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 8\n    key_column: tenant_id",
        )
        .replace("strict_transaction_order", "partitioned_scale_mode");
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    config.validate().expect("valid config");
    assert!(config
        .explain()
        .contains("manifest and commit marker barriers preserving transaction completeness"));
}
