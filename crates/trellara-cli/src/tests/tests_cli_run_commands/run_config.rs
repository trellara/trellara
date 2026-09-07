use super::*;

#[test]
fn run_config_validation_accepts_local_stream_with_target() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse");

    validate_local_run_config(&config).expect("valid local run config");
}

#[test]
fn run_config_validation_rejects_kafka_stream() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");

    assert!(matches!(
        validate_local_run_config(&config),
        Err(CliError::InvalidConfig(message))
            if message.contains("requires stream.kind: local")
    ));
}

#[test]
fn run_config_validation_requires_target() {
    let yaml = local_stream_yaml().replace(
        "target:\n  database_url: postgresql://trellara:trellara@localhost:55433/trellara_target\n",
        "",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    assert!(matches!(
        validate_local_run_config(&config),
        Err(CliError::InvalidConfig(message))
            if message == "target.database_url is required for trellara run"
    ));
}
