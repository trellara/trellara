use super::*;

#[tokio::test]
async fn quickstart_check_command_renders_readiness() {
    let root = std::env::temp_dir().join(format!(
        "trellara-quickstart-command-{}",
        std::process::id()
    ));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create quickstart temp dir");
    fs::write(&config_path, local_stream_yaml()).expect("write quickstart config");

    let output = execute(Cli {
        command: Command::Quickstart(QuickstartArgs {
            config: config_path,
            check: true,
            format: QuickstartOutputFormat::Json,
        }),
    })
    .await
    .expect("quickstart check output");

    assert!(output.contains("\"ready\": true"));
    assert!(output.contains("\"estimated_minutes\": 8"));
    assert!(output.contains("\"time_budget_minutes\": 10"));
    assert!(output.contains("\"evidence_bundle\": \"target/trellara-quickstart-evidence\""));
    assert!(output.contains("trellara status --config"));
    assert!(output.contains("--view diagnostics"));
    assert!(output.contains("\"local_run_ready\""));
    assert!(output.contains("\"transaction_boundary\""));
    assert!(output.contains("\"capture_spill_boundary\""));
    assert!(output.contains("trellara check --config"));
    assert!(output.contains("--format text"));
    assert!(output.contains("trellara run --local --verify --format text --config"));
    assert!(output.contains("trellara status --config"));
    assert!(output.contains("--view report"));
    assert!(output.contains("trellara pilot-package --config"));
    assert!(output.contains("target/trellara-quickstart-evidence"));

    fs::remove_dir_all(root).expect("remove quickstart temp dir");
}

#[tokio::test]
async fn quickstart_check_command_renders_text_readiness() {
    let root = std::env::temp_dir().join(format!(
        "trellara-quickstart-text-command-{}",
        std::process::id()
    ));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create quickstart temp dir");
    fs::write(&config_path, local_stream_yaml()).expect("write quickstart config");

    let output = execute(Cli {
        command: Command::Quickstart(QuickstartArgs {
            config: config_path.clone(),
            check: true,
            format: QuickstartOutputFormat::Text,
        }),
    })
    .await
    .expect("quickstart check text output");

    assert!(output.contains("Trellara quickstart readiness"));
    assert!(output.contains("ready: true"));
    assert!(output.contains("checks: 9/9 passed"));
    assert!(output.contains("time: estimated 8 minutes, budget 10 minutes"));
    assert!(output.contains("evidence_bundle: target/trellara-quickstart-evidence"));
    assert!(output.contains("[pass] transaction_boundary"));
    assert!(output.contains("[pass] capture_spill_boundary"));
    assert!(output.contains("[pass] local_run_ready"));
    assert!(output.contains("next_commands:"));
    assert!(output.contains(&format!(
        "trellara check --config {} --format text",
        config_path.display()
    )));
    assert!(output.contains(&format!(
        "trellara status --config {} --view report --format text",
        config_path.display()
    )));
    assert!(output.contains(&format!(
        "trellara pilot-package --config {} --output target/trellara-quickstart-evidence",
        config_path.display()
    )));
    assert!(output.contains(&format!(
        "recovery: trellara status --config {} --view diagnostics --format text",
        config_path.display()
    )));

    fs::remove_dir_all(root).expect("remove quickstart temp dir");
}
