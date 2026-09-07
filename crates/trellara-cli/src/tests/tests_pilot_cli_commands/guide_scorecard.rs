use super::*;

#[test]
fn cli_parses_pilot_guide_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot",
        "guide",
        "--config",
        "examples/retail-fleet/local.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Pilot {
            command: PilotCommand::Guide(PilotGuideArgs {
                format: PilotGuideOutputFormat::Json,
                ..
            })
        }
    ));
}

#[test]
fn cli_parses_pilot_guide_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot",
        "guide",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Pilot {
            command: PilotCommand::Guide(PilotGuideArgs {
                format: PilotGuideOutputFormat::Text,
                ..
            })
        }
    ));
}

#[test]
fn cli_parses_pilot_guide_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot-guide",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PilotGuide(PilotGuideArgs {
            format: PilotGuideOutputFormat::Text,
            ..
        })
    ));
}

#[test]
fn cli_parses_pilot_scorecard_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot",
        "scorecard",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Pilot {
            command: PilotCommand::Scorecard(PilotScorecardArgs {
                format: PilotGuideOutputFormat::Text,
                ..
            })
        }
    ));
}

#[test]
fn cli_parses_pilot_scorecard_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot-scorecard",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PilotScorecard(PilotScorecardArgs {
            format: PilotGuideOutputFormat::Text,
            ..
        })
    ));
}
