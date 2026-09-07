use super::*;

fn production_yaml(source_env: &str, target_env: &str) -> String {
    format!(
        r#"config_version: 2
environment: production
source:
  id: prod-source
  database_url:
    source: environment_variable
    name: {source_env}
  publication: trellara_prod
  slot: trellara_prod_slot
dataset:
  id: prod-dataset
  mode: strict_transaction_order
  tables:
    - schema: public
      name: events
stream:
  kind: kafka
  bootstrap_servers: broker-1:9093,broker-2:9093,broker-3:9093
  topic: trellara.prod.events
  profile:
    kind: production
    contract:
      contract_version: 1
      ca_certificate:
        source: file
        path: /run/secrets/kafka-ca.pem
      authentication:
        kind: sasl_tls
        mechanism: scram_sha512
        username:
          source: environment_variable
          name: TRELLARA_KAFKA_USERNAME
        password:
          source: environment_variable
          name: TRELLARA_KAFKA_PASSWORD
      replication_factor: 3
      min_insync_replicas: 2
      producer_acks_all: true
      producer_idempotence: true
      consumer_auto_commit: false
      store_offset_after_apply: true
      synchronous_offset_commit: true
target:
  database_url:
    source: environment_variable
    name: {target_env}
"#
    )
}

#[test]
fn legacy_config_migrates_to_v2_development_schema() {
    let migration = TrellaraConfig::migrate_yaml(STRICT_YAML, "legacy").expect("migration");
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "legacy").expect("config");

    assert_eq!(migration.from_version, 1);
    assert_eq!(migration.to_version, CURRENT_CONFIG_VERSION);
    assert!(migration.migrated);
    assert!(migration.yaml.contains("environment: development"));
    assert!(migration.yaml.contains("profile:\n    kind: development"));
    assert_eq!(config.environment, ConfigurationEnvironment::Development);
    config.validate().expect("legacy development config");
}

#[test]
fn migration_is_idempotent_and_rejects_future_versions() {
    let first = TrellaraConfig::migrate_yaml(STRICT_YAML, "legacy").expect("first");
    let second = TrellaraConfig::migrate_yaml(&first.yaml, "v2").expect("second");
    assert!(!second.migrated);
    assert_eq!(first.yaml, second.yaml);

    let error = TrellaraConfig::from_yaml("config_version: 99\n", "future")
        .expect_err("unsupported version");
    assert!(error.to_string().contains("unsupported config_version 99"));
}

#[test]
fn v2_kafka_schema_requires_an_explicit_profile() {
    let yaml = format!(
        "config_version: 2\nenvironment: development\n{}",
        STRICT_YAML
    );
    let error = TrellaraConfig::from_yaml(&yaml, "missing-profile").expect_err("profile");
    assert!(error.to_string().contains("missing field `profile`"));
}

#[test]
fn production_schema_resolves_database_refs_and_selects_kafka_contract() {
    let suffix = std::process::id();
    let source_env = format!("TRELLARA_CONFIG_SOURCE_{suffix}");
    let target_env = format!("TRELLARA_CONFIG_TARGET_{suffix}");
    std::env::set_var(
        &source_env,
        "postgresql://source-user:source-secret@db/source",
    );
    std::env::set_var(
        &target_env,
        "postgresql://target-user:target-secret@db/target",
    );

    let config =
        TrellaraConfig::from_yaml(&production_yaml(&source_env, &target_env), "production")
            .expect("production config");
    config.validate().expect("production validation");
    assert!(config.source.database_url.is_reference());
    #[cfg(feature = "kafka")]
    {
        let publisher = config.to_kafka_publisher_config().expect("publisher");
        let consumer = config.to_kafka_consumer_config().expect("consumer");
        assert!(publisher.security.is_some());
        assert_eq!(
            publisher
                .topology
                .expect("topology")
                .minimum_replication_factor,
            3
        );
        assert_eq!(
            consumer
                .topology
                .expect("topology")
                .minimum_in_sync_replicas,
            2
        );
    }
    std::env::remove_var(source_env);
    std::env::remove_var(target_env);
}

#[test]
fn production_schema_rejects_inline_database_credentials() {
    let yaml = production_yaml("TRELLARA_INLINE_SOURCE", "TRELLARA_INLINE_TARGET").replace(
        "database_url:\n    source: environment_variable\n    name: TRELLARA_INLINE_SOURCE",
        "database_url: postgresql://user:secret@db/source",
    );
    std::env::set_var("TRELLARA_INLINE_TARGET", "postgresql://db/target");
    let config = TrellaraConfig::from_yaml(&yaml, "inline").expect("parse");
    let error = config.validate().expect_err("inline secret");
    assert!(error
        .to_string()
        .contains("must use environment_variable or file"));
    assert!(!error.to_string().contains("secret@db"));
    std::env::remove_var("TRELLARA_INLINE_TARGET");
}

#[test]
fn production_schema_rejects_unsafe_kafka_quorum() {
    let source_env = "TRELLARA_QUORUM_SOURCE";
    let target_env = "TRELLARA_QUORUM_TARGET";
    std::env::set_var(source_env, "postgresql://db/source");
    std::env::set_var(target_env, "postgresql://db/target");
    let yaml = production_yaml(source_env, target_env).replace(
        "replication_factor: 3\n      min_insync_replicas: 2",
        "replication_factor: 1\n      min_insync_replicas: 1",
    );
    let config = TrellaraConfig::from_yaml(&yaml, "quorum").expect("parse");
    assert!(config
        .validate()
        .expect_err("quorum")
        .to_string()
        .contains("replication_factor"));
    std::env::remove_var(source_env);
    std::env::remove_var(target_env);
}

#[test]
fn missing_database_secret_error_redacts_reference_name() {
    let name = format!("TRELLARA_MISSING_DATABASE_URL_{}", std::process::id());
    std::env::remove_var(&name);
    let yaml = STRICT_YAML.replace(
        "database_url: postgresql://trellara:trellara@localhost:55432/trellara_source",
        &format!("database_url:\n    source: environment_variable\n    name: {name}"),
    );
    let error = TrellaraConfig::from_yaml(&yaml, "missing").expect_err("missing secret");
    assert!(!error.to_string().contains(&name));
}

#[test]
fn database_url_file_reference_uses_absolute_utf8_secret_mount() {
    let path = std::env::temp_dir().join(format!(
        "trellara-config-secret-{}-{}.txt",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&path, "postgresql://mounted:secret@db/source\n").expect("write secret");
    let yaml = STRICT_YAML.replace(
        "database_url: postgresql://trellara:trellara@localhost:55432/trellara_source",
        &format!(
            "database_url:\n    source: file\n    path: {}",
            path.display()
        ),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "file-secret").expect("file secret");

    assert_eq!(config.source.database_url.origin(), SecretOrigin::File);
    assert_eq!(
        config.source.database_url.expose(),
        "postgresql://mounted:secret@db/source"
    );
    std::fs::remove_file(path).expect("remove secret");
}

#[test]
fn production_environment_rejects_local_stream_schema() {
    let source_env = "TRELLARA_LOCAL_PRODUCTION_SOURCE";
    let target_env = "TRELLARA_LOCAL_PRODUCTION_TARGET";
    std::env::set_var(source_env, "postgresql://db/source");
    std::env::set_var(target_env, "postgresql://db/target");
    let yaml = local_stream_yaml()
        .replace(
            "database_url: postgresql://trellara:trellara@localhost:55432/trellara_source",
            &format!("database_url:\n    source: environment_variable\n    name: {source_env}"),
        )
        .replace(
            "database_url: postgresql://trellara:trellara@localhost:55433/trellara_target",
            &format!("database_url:\n    source: environment_variable\n    name: {target_env}"),
        )
        .replacen(
            "source:\n",
            "config_version: 2\nenvironment: production\n\nsource:\n",
            1,
        );
    let config = TrellaraConfig::from_yaml(&yaml, "production-local").expect("parse");
    let error = config.validate().expect_err("local production");
    assert!(error
        .to_string()
        .contains("requires Kafka stream.profile.kind=production"));
    std::env::remove_var(source_env);
    std::env::remove_var(target_env);
}

#[test]
fn debug_and_redacted_reports_never_include_secret_material() {
    let secret = "postgresql://admin:never-log-this@db/prod";
    let config = TrellaraConfig::from_yaml(
        &STRICT_YAML.replace(
            "postgresql://trellara:trellara@localhost:55432/trellara_source",
            secret,
        ),
        "debug",
    )
    .expect("config");
    let debug = format!("{config:?}");
    let migration_debug = format!(
        "{:?}",
        TrellaraConfig::migrate_yaml(STRICT_YAML, "migration").expect("migration")
    );
    let redacted = TrellaraConfig::redacted_yaml(STRICT_YAML, "report").expect("redacted");
    let production_redacted = TrellaraConfig::redacted_yaml(
        &production_yaml("TRELLARA_REPORT_SOURCE", "TRELLARA_REPORT_TARGET"),
        "production-report",
    )
    .expect("production redaction");

    assert!(!debug.contains("never-log-this"));
    assert!(!migration_debug.contains("trellara:trellara"));
    assert!(!redacted.contains("trellara:trellara"));
    assert!(redacted.contains("database_url: <redacted>"));
    assert!(!production_redacted.contains("TRELLARA_KAFKA_PASSWORD"));
    assert!(!production_redacted.contains("/run/secrets/kafka-ca.pem"));
    assert!(production_redacted.contains("<redacted:environment_variable>"));
}

#[test]
fn config_commands_parse_as_public_operator_surface() {
    let migrate = Cli::try_parse_from(["trellara", "config", "migrate", "--config", "flow.yml"])
        .expect("migrate command");
    let redact = Cli::try_parse_from(["trellara", "config", "redact", "--config", "flow.yml"])
        .expect("redact command");
    assert!(matches!(
        migrate.command,
        Command::Config {
            command: ConfigurationCommand::Migrate(_)
        }
    ));
    assert!(matches!(
        redact.command,
        Command::Config {
            command: ConfigurationCommand::Redact(_)
        }
    ));
}

#[test]
fn config_commands_migrate_and_redact_without_resolving_references() {
    let path = std::env::temp_dir().join(format!(
        "trellara-config-command-{}.yml",
        std::process::id()
    ));
    std::fs::write(&path, STRICT_YAML).expect("write config");
    let args = ConfigArgs {
        config: path.clone(),
    };
    let migrated = execute_configuration_command(ConfigurationCommand::Migrate(args.clone()))
        .expect("migrate");
    let redacted =
        execute_configuration_command(ConfigurationCommand::Redact(args)).expect("redact");

    assert!(migrated.contains("config_version: 2"));
    assert!(migrated.contains("kind: development"));
    assert!(!redacted.contains("trellara:trellara"));
    assert!(redacted.contains("database_url: <redacted>"));
    std::fs::remove_file(path).expect("remove config");
}

#[test]
fn checked_in_production_example_matches_current_schema() {
    let old_source = std::env::var_os("TRELLARA_SOURCE_DATABASE_URL");
    let old_target = std::env::var_os("TRELLARA_TARGET_DATABASE_URL");
    std::env::set_var("TRELLARA_SOURCE_DATABASE_URL", "postgresql://db/source");
    std::env::set_var("TRELLARA_TARGET_DATABASE_URL", "postgresql://db/target");
    let config = TrellaraConfig::from_path(&workspace_path("examples/retail-fleet/production.yml"))
        .expect("production example");

    config.validate().expect("production example contract");
    assert_eq!(config.environment, ConfigurationEnvironment::Production);
    assert!(matches!(
        config.stream,
        StreamConfig::Kafka {
            profile: KafkaProfile::Production { .. },
            ..
        }
    ));
    restore_environment("TRELLARA_SOURCE_DATABASE_URL", old_source);
    restore_environment("TRELLARA_TARGET_DATABASE_URL", old_target);
}

fn restore_environment(name: &str, previous: Option<std::ffi::OsString>) {
    if let Some(value) = previous {
        std::env::set_var(name, value);
    } else {
        std::env::remove_var(name);
    }
}
