use super::*;

fn env(name: &str) -> KafkaSecretRef {
    KafkaSecretRef::EnvironmentVariable {
        name: name.to_string(),
    }
}

fn production_contract() -> KafkaProductionContract {
    KafkaProductionContract::sasl_tls(
        env("TRELLARA_KAFKA_CA"),
        KafkaSaslMechanism::ScramSha512,
        env("TRELLARA_KAFKA_USERNAME"),
        env("TRELLARA_KAFKA_PASSWORD"),
    )
}

#[test]
fn production_contract_uses_quorum_and_post_apply_progress_defaults() {
    let contract = production_contract();

    assert_eq!(contract.replication_factor, 3);
    assert_eq!(contract.min_insync_replicas, 2);
    assert!(contract.producer_acks_all);
    assert!(contract.producer_idempotence);
    assert!(!contract.consumer_auto_commit);
    assert!(contract.store_offset_after_apply);
    assert!(contract.synchronous_offset_commit);
    assert!(contract.validate().is_ok());
}

#[test]
fn production_contract_rejects_single_broker_durability() {
    let mut contract = production_contract();
    contract.replication_factor = 1;
    contract.min_insync_replicas = 1;

    assert!(matches!(
        contract.validate(),
        Err(KafkaStreamError::InvalidProductionContract {
            field: "replication_factor",
            ..
        })
    ));
}

#[test]
fn production_contract_rejects_progress_before_apply() {
    let mut contract = production_contract();
    contract.consumer_auto_commit = true;

    assert!(matches!(
        contract.validate(),
        Err(KafkaStreamError::InvalidProductionContract {
            field: "consumer_auto_commit",
            ..
        })
    ));
}

#[test]
fn production_contract_rejects_inline_or_blank_secret_locations() {
    let mut contract = production_contract();
    contract.ca_certificate = KafkaSecretRef::File {
        path: " ".to_string(),
    };

    assert!(matches!(
        contract.validate(),
        Err(KafkaStreamError::InvalidProductionContract {
            field: "ca_certificate",
            ..
        })
    ));
}

#[test]
fn production_contract_rejects_relative_secret_files() {
    let mut contract = production_contract();
    contract.ca_certificate = KafkaSecretRef::File {
        path: "secrets/ca.pem".to_string(),
    };

    assert!(matches!(
        contract.validate(),
        Err(KafkaStreamError::InvalidProductionContract {
            field: "ca_certificate",
            ..
        })
    ));
}

#[test]
fn production_contract_rejects_unknown_versions() {
    let mut contract = production_contract();
    contract.contract_version += 1;

    assert!(matches!(
        contract.validate(),
        Err(KafkaStreamError::UnsupportedProductionContractVersion { .. })
    ));
}
