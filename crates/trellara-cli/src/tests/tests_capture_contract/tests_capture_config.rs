use super::*;

#[test]
fn maps_yaml_to_capture_config() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let capture = config.to_capture_config(true).expect("capture config");

    assert_eq!(capture.source_id, "local-source");
    assert_eq!(capture.database_id, "postgres");
    assert_eq!(capture.dataset_id, "retail-sales");
    assert_eq!(capture.publication_name, "trellara_retail");
    assert_eq!(capture.slot_name, "trellara_retail_slot");
    assert_eq!(capture.tables.len(), 1);
    assert!(capture.create_if_missing);
    assert_eq!(
        capture.stream_spill_threshold_changes,
        trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES
    );
    assert_eq!(capture.stream_spill_dir, None);
    assert_eq!(
        capture.pgoutput.protocol_version,
        trellara_pg_capture::DEFAULT_PGOUTPUT_PROTOCOL_VERSION
    );
    assert_eq!(
        capture.pgoutput.streaming,
        trellara_pg_capture::DEFAULT_PGOUTPUT_STREAMING
    );
}

#[test]
fn maps_yaml_stream_spill_settings_to_capture_config() {
    let yaml = STRICT_YAML.replace(
        "  slot: trellara_retail_slot",
        "  slot: trellara_retail_slot\n  stream_spill_threshold_changes: 7\n  stream_spill_dir: /tmp/trellara-spill",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let capture = config.to_capture_config(true).expect("capture config");

    assert_eq!(config.source.stream_spill_threshold_changes, Some(7));
    assert_eq!(capture.stream_spill_threshold_changes, 7);
    assert_eq!(
        capture.stream_spill_dir.expect("spill dir"),
        PathBuf::from("/tmp/trellara-spill")
    );
}

#[test]
fn maps_yaml_pgoutput_protocol_settings_to_capture_config() {
    let yaml = STRICT_YAML.replace(
        "  slot: trellara_retail_slot",
        "  slot: trellara_retail_slot\n  pgoutput:\n    protocol_version: 1\n    streaming: false",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let capture = config.to_capture_config(true).expect("capture config");

    assert_eq!(config.source.pgoutput.protocol_version, 1);
    assert!(!config.source.pgoutput.streaming);
    assert_eq!(capture.pgoutput.protocol_version, 1);
    assert!(!capture.pgoutput.streaming);
}

#[test]
fn pgoutput_streaming_requires_protocol_v2_in_flow_config() {
    let yaml = STRICT_YAML.replace(
        "  slot: trellara_retail_slot",
        "  slot: trellara_retail_slot\n  pgoutput:\n    protocol_version: 1\n    streaming: true",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message.contains("pgoutput.streaming requires protocol_version 2")
    ));
}

#[test]
fn test_decoding_capture_ignores_pgoutput_protocol_settings() {
    let yaml = STRICT_YAML.replace(
        "  database_url: postgresql://trellara:trellara@localhost:55432/trellara_source",
        "  database_url: postgresql://trellara:trellara@localhost:55432/trellara_source\n  capture: test_decoding\n  pgoutput:\n    protocol_version: 1\n    streaming: true",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    config.validate().expect("test_decoding validation");
}

#[test]
fn stream_spill_threshold_must_be_positive() {
    let yaml = STRICT_YAML.replace(
        "  slot: trellara_retail_slot",
        "  slot: trellara_retail_slot\n  stream_spill_threshold_changes: 0",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message.contains("stream_spill_threshold_changes")
    ));
}

#[test]
fn stream_spill_threshold_has_upper_bound() {
    let yaml = STRICT_YAML.replace(
        "  slot: trellara_retail_slot",
        &format!(
            "  slot: trellara_retail_slot\n  stream_spill_threshold_changes: {}",
            trellara_pg_capture::MAX_STREAM_SPILL_THRESHOLD_CHANGES + 1
        ),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        config.validate(),
        Err(CliError::InvalidConfig(message))
            if message.contains("stream_spill_threshold_changes")
                && message.contains("at most")
    ));
}
