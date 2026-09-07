use super::*;

#[test]
fn cli_parses_run_command_with_bounded_defaults() {
    let cli =
        Cli::try_parse_from(["trellara", "run", "--config", "trellara.yml"]).expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Run(RunArgs {
            config,
            local: false,
            format: QuickstartOutputFormat::Json,
            skip_snapshot: false,
            snapshot_run_id,
            max_transactions: 100,
            max_messages: 100,
            ..
        }) if config.as_path() == Path::new("trellara.yml") && snapshot_run_id == "local-run-snapshot"
    ));
}

#[test]
fn cli_parses_run_local_public_loop() {
    let cli = Cli::try_parse_from([
        "trellara",
        "run",
        "--local",
        "--verify",
        "--format",
        "text",
        "--config",
        "trellara.yml",
        "--snapshot-run-id",
        "pilot-snapshot",
        "--max-transactions",
        "10",
        "--max-messages",
        "20",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Run(RunArgs {
            config,
            local: true,
            verify: true,
            format: QuickstartOutputFormat::Text,
            skip_snapshot: false,
            snapshot_run_id,
            max_transactions: 10,
            max_messages: 20,
            ..
        }) if config.as_path() == Path::new("trellara.yml") && snapshot_run_id == "pilot-snapshot"
    ));
}

#[test]
fn cli_parses_run_skip_snapshot_for_debug_loops() {
    let cli = Cli::try_parse_from([
        "trellara",
        "run",
        "--config",
        "trellara.yml",
        "--skip-snapshot",
        "--max-transactions",
        "10",
        "--max-messages",
        "20",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Run(RunArgs {
            skip_snapshot: true,
            max_transactions: 10,
            max_messages: 20,
            ..
        })
    ));
}

#[test]
fn cli_parses_demo_command_as_local_run_alias() {
    let cli = Cli::try_parse_from([
        "trellara",
        "demo",
        "--config",
        "trellara.yml",
        "--max-transactions",
        "10",
        "--max-messages",
        "20",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Demo(RunArgs {
            config,
            max_transactions: 10,
            max_messages: 20,
            ..
        }) if config.as_path() == Path::new("trellara.yml")
    ));
}

#[test]
fn cli_parses_run_command_with_explicit_bounds() {
    let cli = Cli::try_parse_from([
        "trellara",
        "run",
        "--config",
        "trellara.yml",
        "--max-transactions",
        "10",
        "--max-messages",
        "20",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Run(RunArgs {
            max_transactions: 10,
            max_messages: 20,
            ..
        })
    ));
}
