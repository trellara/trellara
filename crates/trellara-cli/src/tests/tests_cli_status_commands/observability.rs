use super::*;

#[test]
fn cli_parses_report_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "report",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Report(_)));
}

#[test]
fn cli_parses_alerts_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "alerts",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Alerts(_)));
}

#[test]
fn cli_parses_dashboard_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "dashboard",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Dashboard(_)));
}

#[test]
fn cli_parses_metrics_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "metrics",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Metrics(_)));
}

#[test]
fn cli_parses_diagnostics_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "diagnostics",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::Diagnostics(_)));
}
