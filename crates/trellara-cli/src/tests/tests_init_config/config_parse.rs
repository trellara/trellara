use super::*;

#[test]
fn parses_and_validates_strict_config() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    config.validate().expect("valid config");
    assert_eq!(config.source.id, "local-source");
    assert_eq!(config.source.capture, SourceCaptureKind::PgOutput);
    assert_eq!(config.dataset.mode, DatasetMode::StrictTransactionOrder);
    assert_eq!(
        config.dataset.unknown_table_policy,
        UnknownTablePolicy::Reject
    );
    assert_eq!(config.dataset.tables.len(), 1);
}

#[test]
fn parses_unknown_table_policy() {
    let yaml = STRICT_YAML.replace(
        "  mode: strict_transaction_order",
        "  mode: strict_transaction_order\n  unknown_table_policy: allow_compatible",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert_eq!(
        config.dataset.unknown_table_policy,
        UnknownTablePolicy::AllowCompatible
    );
    config.validate().expect("valid config");
}

#[test]
fn parses_explicit_test_decoding_capture_kind() {
    let yaml = STRICT_YAML.replace(
            "  database_url: postgresql://trellara:trellara@localhost:55432/trellara_source",
            "  database_url: postgresql://trellara:trellara@localhost:55432/trellara_source\n  capture: test_decoding",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert_eq!(config.source.capture, SourceCaptureKind::TestDecoding);
}

#[test]
fn parses_source_wal_retention_warning_policy() {
    let yaml = STRICT_YAML.replace(
        "  slot: trellara_retail_slot",
        "  slot: trellara_retail_slot\n  wal_retention_warn_bytes: 1048576",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    config.validate().expect("valid retention policy");
    assert_eq!(config.source.wal_retention_warn_bytes, Some(1_048_576));
}

#[test]
fn source_wal_retention_warning_policy_must_be_positive() {
    let yaml = STRICT_YAML.replace(
        "  slot: trellara_retail_slot",
        "  slot: trellara_retail_slot\n  wal_retention_warn_bytes: 0",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message == "source.wal_retention_warn_bytes must be greater than zero"
    ));
}

#[test]
fn capture_kind_selects_expected_slot_plugin() {
    assert_eq!(SourceCaptureKind::PgOutput.expected_plugin(), "pgoutput");
    assert_eq!(
        SourceCaptureKind::TestDecoding.expected_plugin(),
        "test_decoding"
    );
}
