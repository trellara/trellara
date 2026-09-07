use super::*;

#[tokio::test]
async fn version_command_reports_package_version() {
    let cli = Cli::try_parse_from(["trellara", "version"]).expect("cli parse");

    assert_eq!(
        execute(cli).await.expect("execute"),
        format!("trellara {}", env!("CARGO_PKG_VERSION"))
    );
}

#[tokio::test]
async fn dev_up_dry_run_reports_no_broker_stack_command() {
    let cli = Cli::try_parse_from(["trellara", "dev", "up", "--dry-run"]).expect("cli parse");
    let output = execute(cli).await.expect("execute");

    assert!(
        output.contains("\"command\": \"docker compose up -d source-postgres target-postgres\"")
    );
    assert!(output.contains("source-postgres"));
    assert!(output.contains("target-postgres"));
    assert!(!output.contains("redpanda"));
    assert!(output.contains("\"dry_run\": true"));
    assert!(output.contains("\"executed\": false"));
}

#[tokio::test]
async fn dev_up_dry_run_can_include_kafka_or_runtime_stack() {
    let kafka = execute(
        Cli::try_parse_from(["trellara", "dev", "up", "--dry-run", "--with-kafka"])
            .expect("cli parse"),
    )
    .await
    .expect("execute");
    assert!(kafka.contains("redpanda"));
    assert!(!kafka.contains("trellara-relay"));

    let runtime = execute(
        Cli::try_parse_from(["trellara", "dev", "up", "--dry-run", "--runtime"])
            .expect("cli parse"),
    )
    .await
    .expect("execute");
    assert!(runtime.contains("docker compose --profile runtime up -d"));
    assert!(runtime.contains("redpanda"));
    assert!(runtime.contains("trellara-relay"));
    assert!(runtime.contains("trellara-applier"));
    let runtime_json: serde_json::Value =
        serde_json::from_str(&runtime).expect("runtime dry-run json");
    assert_eq!(
        runtime_json["services"],
        serde_json::json!([
            "source-postgres",
            "target-postgres",
            "redpanda",
            "trellara-relay",
            "trellara-applier"
        ])
    );
}

#[tokio::test]
async fn dev_down_and_logs_support_dry_run() {
    let down = execute(
        Cli::try_parse_from(["trellara", "dev", "down", "--dry-run", "--volumes"])
            .expect("cli parse"),
    )
    .await
    .expect("execute");
    assert!(down.contains("\"command\": \"docker compose down -v\""));
    assert!(down.contains("\"action\": \"down\""));

    let logs = execute(
        Cli::try_parse_from(["trellara", "dev", "logs", "--dry-run", "--runtime"])
            .expect("cli parse"),
    )
    .await
    .expect("execute");
    assert!(logs.contains("\"command\": \"docker compose --profile runtime logs"));
    assert!(logs.contains("trellara-relay"));
}
