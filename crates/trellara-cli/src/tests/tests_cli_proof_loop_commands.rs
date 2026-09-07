use super::*;

#[test]
fn cli_parses_contract_test_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "contract",
        "test",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Contract {
            command: ContractCommand::Test(_)
        }
    ));
}

#[test]
fn cli_parses_contract_test_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "contract-test",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::ContractTest(_)));
}

#[test]
fn cli_parses_chaos_run_command() {
    let cli = Cli::try_parse_from(["trellara", "chaos", "run"]).expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Chaos {
            command: ChaosCommand::Run
        }
    ));
}

#[test]
fn cli_parses_chaos_report_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "chaos",
        "report",
        "--output",
        "docs/correctness-report.html",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Chaos {
            command: ChaosCommand::Report(ChaosReportArgs { output, .. })
        } if output.as_path() == Path::new("docs/correctness-report.html")
    ));
}

#[test]
fn cli_parses_quickstart_text_check_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "quickstart",
        "--config",
        "trellara.yml",
        "--check",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Quickstart(QuickstartArgs {
            config,
            check: true,
            format: QuickstartOutputFormat::Text,
        }) if config.as_path() == Path::new("trellara.yml")
    ));
}

#[test]
fn cli_parses_preflight_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "preflight",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Preflight(_)));
}

#[test]
fn cli_parses_apply_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "apply",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--max-messages",
        "10",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Apply(ApplyArgs {
            max_messages: 10,
            ..
        })
    ));
}
