use super::*;

#[test]
fn starting_health_uses_contract_defaults() {
    let health = RuntimeHealth::starting(RuntimeService::Relay, "source-a", "orders");

    assert_eq!(health.contract_version, RUNTIME_CONTRACT_VERSION);
    assert_eq!(health.phase, RuntimePhase::Starting);
    assert_eq!(health.readiness, RuntimeReadiness::NotReady);
    assert!(!health.accepting_work);
    assert!(health.validate().is_ok());
}

#[test]
fn running_readiness_controls_work_admission() {
    let mut health = RuntimeHealth::starting(RuntimeService::Applier, "source-a", "orders");
    health.phase = RuntimePhase::Running;
    health.readiness = RuntimeReadiness::Ready;
    health.accepting_work = true;
    assert!(health.validate().is_ok());

    health.readiness = RuntimeReadiness::Backpressured;
    health.reason_code = Some("queue_full".to_string());
    assert!(matches!(
        health.validate(),
        Err(RuntimeContractError::InvalidState { .. })
    ));
    health.accepting_work = false;
    assert!(health.validate().is_ok());
}

#[test]
fn terminal_phases_cannot_accept_work() {
    let mut health = RuntimeHealth::starting(RuntimeService::IcebergWriter, "source-a", "orders");
    health.phase = RuntimePhase::Failed;
    health.readiness = RuntimeReadiness::Blocked;
    health.accepting_work = true;
    health.reason_code = Some("catalog_conflict".to_string());

    assert!(matches!(
        health.validate(),
        Err(RuntimeContractError::InvalidState { .. })
    ));
}

#[test]
fn runtime_contract_rejects_unknown_versions_and_blank_identity() {
    let mut health = RuntimeHealth::starting(RuntimeService::Relay, "source-a", "orders");
    health.contract_version += 1;
    assert!(matches!(
        health.validate(),
        Err(RuntimeContractError::UnsupportedVersion { .. })
    ));

    health.contract_version = RUNTIME_CONTRACT_VERSION;
    health.source_id = " ".to_string();
    assert!(matches!(
        health.validate(),
        Err(RuntimeContractError::InvalidField {
            field: "source_id",
            ..
        })
    ));
}

#[test]
fn unhealthy_readiness_requires_a_stable_reason_code() {
    let mut health = RuntimeHealth::starting(RuntimeService::Relay, "source-a", "orders");
    health.phase = RuntimePhase::Running;
    health.readiness = RuntimeReadiness::Degraded;
    health.accepting_work = true;
    assert!(matches!(
        health.validate(),
        Err(RuntimeContractError::InvalidField {
            field: "reason_code",
            ..
        })
    ));

    health.reason_code = Some("broker unavailable".to_string());
    assert!(health.validate().is_err());
    health.reason_code = Some("broker_unavailable".to_string());
    assert!(health.validate().is_ok());
}

#[test]
fn support_matrix_distinguishes_external_and_native_postgres() {
    assert_eq!(SUPPORTED_EXTERNAL_POSTGRES_MAJORS, [16, 17, 18]);
    assert_eq!(SUPPORTED_NATIVE_POSTGRES_MAJORS, [17, 18]);
    assert_eq!(
        RELEASE_ARCHITECTURES,
        [ReleaseArchitecture::Amd64, ReleaseArchitecture::Arm64]
    );
    assert!(supports_external_postgres_major(16));
    assert!(!supports_native_postgres_major(16));
    assert!(supports_native_postgres_major(17));
    assert!(supports_native_postgres_major(18));
    assert!(!supports_external_postgres_major(19));
}

#[test]
fn runtime_metric_names_and_labels_are_stable() {
    assert_eq!(RUNTIME_METRIC_LIVE, "trellara_runtime_live");
    assert_eq!(RUNTIME_METRIC_READY, "trellara_runtime_ready");
    assert_eq!(
        RUNTIME_METRIC_LABELS,
        ["service", "source_id", "dataset_id"]
    );
    assert_eq!(
        [
            ReleasePackageFormat::TarGz,
            ReleasePackageFormat::Deb,
            ReleasePackageFormat::Rpm,
            ReleasePackageFormat::OciImage,
        ]
        .len(),
        4
    );
}
