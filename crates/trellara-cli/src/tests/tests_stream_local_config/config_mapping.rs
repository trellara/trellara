use super::*;

#[test]
#[cfg(feature = "kafka")]
fn maps_yaml_to_kafka_consumer_config() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let consumer = config.to_kafka_consumer_config().expect("consumer config");

    assert_eq!(consumer.bootstrap_servers, "localhost:9092");
    assert_eq!(
        consumer.group_id,
        "trellara-applier-local-source-retail-sales"
    );
    assert_eq!(
        consumer.topics,
        vec!["trellara.local-source.retail-sales.strict".to_string()]
    );
    assert!(!consumer.enable_auto_commit);
}

#[test]
fn parses_local_stream_config() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.stream,
        StreamConfig::Local {
            path,
            consumer_group: None,
            durability: LocalStreamDurability::Fsync
        } if path == Path::new("/tmp/trellara-local-stream")
    ));
}

#[test]
fn maps_yaml_to_local_consumer_config() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let consumer = config.to_local_consumer_config().expect("consumer config");

    assert_eq!(consumer.root, PathBuf::from("/tmp/trellara-local-stream"));
    assert_eq!(
        consumer.group_id,
        "trellara-applier-local-source-retail-sales"
    );
    assert_eq!(
        consumer.topics,
        vec!["trellara.local-source.retail-sales.strict".to_string()]
    );
    assert_eq!(consumer.durability, LocalDurability::Fsync);
}

#[test]
fn maps_yaml_to_local_publisher_config() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let publisher = config
        .to_local_publisher_config()
        .expect("publisher config");

    assert_eq!(publisher.root, PathBuf::from("/tmp/trellara-local-stream"));
    assert_eq!(publisher.durability, LocalDurability::Fsync);
}

#[test]
fn maps_buffered_local_durability_to_runtime_configs() {
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        "  path: /tmp/trellara-local-stream\n  durability: buffered",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert_eq!(
        config
            .to_local_publisher_config()
            .expect("publisher")
            .durability,
        LocalDurability::Buffered
    );
    assert_eq!(
        config
            .to_local_consumer_config()
            .expect("consumer")
            .durability,
        LocalDurability::Buffered
    );
}

#[test]
fn explicit_local_consumer_group_overrides_default() {
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        "  path: /tmp/trellara-local-stream\n  consumer_group: local-applier",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let consumer = config.to_local_consumer_config().expect("consumer config");

    assert_eq!(consumer.group_id, "local-applier");
}
