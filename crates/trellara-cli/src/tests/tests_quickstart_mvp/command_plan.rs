use super::*;

#[test]
fn quickstart_summary_lists_no_broker_verified_loop() {
    let summary = QuickstartSummary::from_args(&QuickstartArgs {
        config: PathBuf::from("trellara.yml"),
        check: false,
        format: QuickstartOutputFormat::Json,
    });

    assert_eq!(summary.config, "trellara.yml");
    assert_eq!(summary.command_count, 7);
    assert_eq!(summary.estimated_minutes, QUICKSTART_ESTIMATED_MINUTES);
    assert_eq!(summary.time_budget_minutes, QUICKSTART_TIME_BUDGET_MINUTES);
    assert!(summary.objective.contains("no-broker local verified"));
    assert!(summary.objective.contains("under 10 minutes"));
    assert_eq!(
        summary.evidence_bundle,
        "target/trellara-quickstart-evidence"
    );
    assert_eq!(
        summary.recovery_command,
        "trellara status --config trellara.yml --view diagnostics --format text"
    );
    assert_eq!(summary.commands[0].command, "trellara dev up");
    assert!(summary.commands[0].purpose.contains("no-broker"));
    assert!(summary.commands[1].command.starts_with("trellara init"));
    assert!(summary.commands[1].command.contains("--evaluate"));
    assert_eq!(
        summary.commands[2].command,
        "trellara check --config trellara.yml --format text"
    );
    assert!(summary.commands[2].purpose.contains("read-only"));
    assert_eq!(
        summary.commands[3].command,
        "trellara preflight --config trellara.yml"
    );
    assert!(summary.commands[4]
        .command
        .starts_with("trellara run --local --verify --format text --config trellara.yml"));
    assert!(!summary
        .commands
        .iter()
        .any(|step| step.command == "trellara contract-test --config trellara.yml"));
    assert!(!summary
        .commands
        .iter()
        .any(|step| step.command.starts_with("trellara snapshot")));
    assert!(summary.commands.iter().any(|step| step.command
            == "trellara run --local --verify --format text --config trellara.yml --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"));
    assert!(summary
        .commands
        .iter()
        .any(|step| step.command
            == "trellara status --config trellara.yml --view report --format text"));
    assert!(summary
        .commands
        .iter()
        .any(|step| step.command
            == "trellara pilot-package --config trellara.yml --output target/trellara-quickstart-evidence"));
    assert!(summary
        .operator_reference_commands
        .iter()
        .any(|command| command == "trellara evaluate --config trellara.yml --format text"));
    assert!(summary
        .operator_reference_commands
        .iter()
        .any(|command| command == "trellara pilot-guide --config trellara.yml --format text"));
    assert!(summary
        .operator_reference_commands
        .iter()
        .any(|command| command == "trellara pilot-scorecard --config trellara.yml --format text"));
    assert!(summary
        .operator_reference_commands
        .iter()
        .any(|command| command == "trellara pilot-evidence --config trellara.yml --format text"));
    assert!(summary
        .operator_reference_commands
        .iter()
        .any(|command| command == "trellara chaos report --output docs/correctness-report.html"));
}

#[tokio::test]
async fn quickstart_command_renders_command_plan() {
    let output = execute(Cli {
        command: Command::Quickstart(QuickstartArgs {
            config: PathBuf::from("trellara.yml"),
            check: false,
            format: QuickstartOutputFormat::Json,
        }),
    })
    .await
    .expect("quickstart output");

    assert!(output.contains("\"objective\""));
    assert!(output.contains("\"estimated_minutes\": 8"));
    assert!(output.contains("\"time_budget_minutes\": 10"));
    assert!(output.contains("\"evidence_bundle\": \"target/trellara-quickstart-evidence\""));
    assert!(output.contains(
        "\"recovery_command\": \"trellara status --config trellara.yml --view diagnostics --format text\""
    ));
    assert!(output.contains("trellara init --source-database-url"));
    assert!(output.contains("trellara check --config trellara.yml --format text"));
    assert!(output.contains("trellara preflight --config trellara.yml"));
    assert!(output.contains("trellara run --local --verify --format text --config trellara.yml"));
    assert!(output.contains("trellara status --config trellara.yml --view report"));
    assert!(output.contains(
        "trellara pilot-package --config trellara.yml --output target/trellara-quickstart-evidence"
    ));
    assert!(output.contains("trellara evaluate --config trellara.yml --format text"));
    assert!(output.contains("trellara contract-test --config trellara.yml"));
    assert!(output.contains("trellara pilot-guide --config trellara.yml --format text"));
    assert!(output.contains("trellara pilot-scorecard --config trellara.yml --format text"));
    assert!(output.contains("trellara pilot-evidence --config trellara.yml --format text"));
}

#[tokio::test]
async fn quickstart_command_renders_text_plan() {
    let output = execute(Cli {
        command: Command::Quickstart(QuickstartArgs {
            config: PathBuf::from("trellara.yml"),
            check: false,
            format: QuickstartOutputFormat::Text,
        }),
    })
    .await
    .expect("quickstart text output");

    assert!(output.contains("Trellara quickstart"));
    assert!(output.contains("time: estimated 8 minutes, budget 10 minutes"));
    assert!(output.contains("evidence_bundle: target/trellara-quickstart-evidence"));
    assert!(output.contains(
        "recovery: trellara status --config trellara.yml --view diagnostics --format text"
    ));
    assert!(output.contains("plan:"));
    assert!(output.contains("trellara init --source-database-url"));
    assert!(output.contains("trellara check --config trellara.yml --format text"));
    assert!(output.contains("trellara preflight --config trellara.yml"));
    assert!(output.contains("trellara run --local --verify --format text --config trellara.yml"));
    assert!(output.contains("trellara status --config trellara.yml --view report --format text"));
    assert!(output.contains(
        "trellara pilot-package --config trellara.yml --output target/trellara-quickstart-evidence"
    ));
    assert!(output.contains("operator_reference:"));
    assert!(output.contains("trellara evaluate --config trellara.yml --format text"));
    assert!(output.contains("trellara contract-test --config trellara.yml"));
    assert!(output.contains("trellara pilot-guide --config trellara.yml --format text"));
    assert!(output.contains("trellara pilot-scorecard --config trellara.yml --format text"));
    assert!(output.contains("transaction boundaries, convergence, and recovery evidence"));
}
