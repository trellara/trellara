use std::time::Duration;

use super::*;

#[test]
fn local_config_uses_safe_defaults() {
    let config = KafkaPublisherConfig::local("localhost:9092");

    assert_eq!(config.bootstrap_servers, "localhost:9092");
    assert!(config.enable_idempotence);
    assert_eq!(config.message_timeout, Duration::from_secs(30));
}

#[test]
fn local_consumer_config_disables_auto_commit() {
    let config = KafkaConsumerConfig::local(
        "localhost:9092",
        "trellara-applier-sales",
        vec!["trellara.source.dataset.strict".to_string()],
    );

    assert_eq!(config.bootstrap_servers, "localhost:9092");
    assert_eq!(config.group_id, "trellara-applier-sales");
    assert_eq!(config.poll_timeout, Duration::from_millis(100));
    assert!(!config.enable_auto_commit);
    assert!(config.validate().is_ok());
}

#[test]
fn consumer_config_requires_topics() {
    let config = KafkaConsumerConfig::local("localhost:9092", "group", Vec::new());

    assert!(matches!(
        config.validate(),
        Err(KafkaStreamError::MissingTopics)
    ));
}

#[test]
fn production_configs_map_security_and_quorum_contract() {
    let env = |name: &str| KafkaSecretRef::EnvironmentVariable {
        name: name.to_string(),
    };
    let contract = KafkaProductionContract::sasl_tls(
        env("TRELLARA_KAFKA_CA"),
        KafkaSaslMechanism::ScramSha512,
        env("TRELLARA_KAFKA_USERNAME"),
        env("TRELLARA_KAFKA_PASSWORD"),
    );

    let publisher = KafkaPublisherConfig::production(
        "broker-1:9093,broker-2:9093,broker-3:9093",
        "relay",
        &contract,
    )
    .expect("production publisher config");
    let consumer = KafkaConsumerConfig::production(
        "broker-1:9093,broker-2:9093,broker-3:9093",
        "applier",
        vec!["events".to_string()],
        &contract,
    )
    .expect("production consumer config");

    assert!(publisher.security.is_some());
    assert_eq!(
        publisher
            .topology
            .as_ref()
            .expect("publisher topology")
            .minimum_broker_count,
        3
    );
    assert!(consumer.security.is_some());
    assert_eq!(
        consumer
            .topology
            .as_ref()
            .expect("consumer topology")
            .minimum_in_sync_replicas,
        2
    );
}

#[test]
fn consumer_rejects_auto_commit_and_unbounded_apply_queue() {
    let mut config =
        KafkaConsumerConfig::local("localhost:9092", "applier", vec!["events".to_string()]);
    config.enable_auto_commit = true;
    assert!(matches!(
        config.validate(),
        Err(KafkaStreamError::InvalidConfiguration {
            field: "enable_auto_commit",
            ..
        })
    ));

    config.enable_auto_commit = false;
    config.max_in_flight_messages = 0;
    assert!(matches!(
        config.validate(),
        Err(KafkaStreamError::InvalidConfiguration {
            field: "max_in_flight_messages",
            ..
        })
    ));
}
