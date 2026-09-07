use super::*;

#[test]
fn cli_parses_validate_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "validate",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Validate(_)));
}

#[test]
fn cli_parses_semantics_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "semantics",
        "--config",
        "examples/retail-fleet/partitioned.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Semantics(_)));
}

#[test]
fn cli_parses_version_command() {
    let cli = Cli::try_parse_from(["trellara", "version"]).expect("cli parse");

    assert!(matches!(cli.command, Command::Version));
}

#[test]
fn cli_parses_check_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "check",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Check(_)));
}

#[test]
fn cli_parses_source_safety_compat_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "source-safety",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::SourceSafety(_)));
}

#[test]
fn cli_help_advertises_public_loop_and_configuration_operations() {
    let mut help = Vec::new();
    Cli::command()
        .write_long_help(&mut help)
        .expect("render cli help");
    let help = String::from_utf8(help).expect("help is utf8");

    assert!(help.contains("PostgreSQL sources are safe for CDC"));
    assert!(help.contains("proves convergence"));
    assert!(help.contains(
        "trellara init -> trellara check -> trellara preflight -> trellara run -> trellara verify -> trellara status"
    ));

    let visible_commands = help_command_names(&help);
    assert_eq!(
        visible_commands,
        vec![
            "check",
            "init",
            "run",
            "fleet",
            "config",
            "lake",
            "preflight",
            "status",
            "verify",
        ]
    );

    for hidden_command in [
        "version",
        "quickstart",
        "source-safety",
        "dev",
        "contract-test",
        "evaluate",
        "evidence-registry",
        "pilot-guide",
        "pilot-scorecard",
        "pilot-evidence",
        "pilot-evidence-check",
        "pilot-evidence-template",
        "pilot-package",
        "chaos",
        "stream",
        "quarantine",
    ] {
        assert!(
            !visible_commands.contains(&hidden_command),
            "default help should not advertise hidden command {hidden_command}"
        );
    }
}

fn help_command_names(help: &str) -> Vec<&str> {
    help.lines()
        .skip_while(|line| line.trim() != "Commands:")
        .skip(1)
        .take_while(|line| !line.trim().is_empty() && line.trim() != "Options:")
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| *name != "help")
        .collect()
}

#[test]
fn cli_parses_flow_validate_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "flow",
        "validate",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Flow {
            command: FlowCommand::Validate(_)
        }
    ));
}

#[test]
fn cli_parses_flow_create_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "flow",
        "create",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Flow {
            command: FlowCommand::Create(_)
        }
    ));
}

#[test]
fn cli_parses_flow_status_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "flow",
        "status",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Flow {
            command: FlowCommand::Status(_)
        }
    ));
}
