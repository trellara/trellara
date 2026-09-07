use super::*;

#[tokio::test]
async fn init_command_writes_valid_config_and_refuses_unforced_overwrite() {
    let config_file = std::env::temp_dir().join(format!(
        "trellara-init-{}-{}.yml",
        std::process::id(),
        "flow"
    ));
    let _ = std::fs::remove_file(&config_file);
    let args = init_args(config_file.clone());
    let output = execute(Cli {
        command: Command::Init(args.clone()),
    })
    .await
    .expect("execute init");

    assert!(output.contains("\"stream_kind\": \"local\""));
    assert!(output.contains("\"evaluation_ready\": false"));
    assert!(output.contains("trellara check --config"));
    assert!(output.contains("trellara preflight --config"));
    assert!(output.contains("trellara run --local --verify --format text --config"));
    assert!(output.contains("trellara verify --config"));
    assert!(output.contains("trellara status --config"));
    assert!(!output.contains("trellara evaluate --config"));
    assert!(!output.contains("trellara pilot-package --config"));
    assert!(!output.contains("trellara bootstrap --config"));
    assert!(!output.contains("trellara relay --config"));
    assert!(!output.contains("trellara apply --config"));
    let generated = TrellaraConfig::from_path(&config_file).expect("generated config");
    generated.validate().expect("valid generated config");
    assert_eq!(
        generated.source.stream_spill_threshold_changes,
        Some(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES)
    );
    assert_eq!(
        generated.source.stream_spill_dir.as_deref(),
        Some(Path::new("./target/trellara-spill"))
    );
    assert_eq!(
        generated
            .dataset
            .strict_chunking
            .as_ref()
            .expect("strict chunking")
            .max_changes_per_chunk,
        1000
    );
    assert!(matches!(
        execute(Cli {
            command: Command::Init(args.clone()),
        })
        .await,
        Err(CliError::InvalidConfig(message)) if message.contains("already exists")
    ));

    let mut forced = args;
    forced.force = true;
    execute(Cli {
        command: Command::Init(forced),
    })
    .await
    .expect("forced overwrite");

    std::fs::remove_file(config_file).expect("remove generated config");
}

#[tokio::test]
async fn init_evaluate_writes_config_and_returns_enterprise_next_steps() {
    let config_file = std::env::temp_dir().join(format!(
        "trellara-init-evaluate-{}-{}.yml",
        std::process::id(),
        unique_test_suffix()
    ));
    let _ = std::fs::remove_file(&config_file);
    let mut args = init_args(config_file.clone());
    args.evaluate = true;

    let output = execute(Cli {
        command: Command::Init(args),
    })
    .await
    .expect("execute init evaluate");

    assert!(output.contains("\"evaluation_ready\": true"));
    assert!(output.contains("trellara quickstart --config"));
    assert!(output.contains("trellara status --config"));
    assert!(output.contains("--view report"));
    assert!(output.contains("trellara evaluate --config"));
    assert!(output.contains("trellara pilot-package --config"));
    let generated = TrellaraConfig::from_path(&config_file).expect("generated config");
    generated.validate().expect("valid generated config");
    assert_eq!(generated.status_mode(), "strict_chunked_transaction_order");

    std::fs::remove_file(config_file).expect("remove generated config");
}
