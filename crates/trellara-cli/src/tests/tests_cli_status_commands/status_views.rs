use super::*;

#[test]
fn cli_parses_status_report_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "status",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--view",
        "report",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Status(StatusArgs {
            view: StatusView::Report,
            format: QuickstartOutputFormat::Text,
            ..
        })
    ));
}

#[test]
fn cli_parses_status_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "status",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Status(StatusArgs {
            view: StatusView::Flow,
            ..
        })
    ));
}

#[test]
fn cli_parses_status_view_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "status",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--view",
        "diagnostics",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Status(StatusArgs {
            view: StatusView::Diagnostics,
            ..
        })
    ));
}
