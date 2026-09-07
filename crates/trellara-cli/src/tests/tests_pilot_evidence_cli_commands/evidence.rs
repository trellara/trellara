use super::*;

#[test]
fn cli_parses_pilot_evidence_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot",
        "evidence",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Pilot {
            command: PilotCommand::Evidence(PilotEvidenceArgs {
                format: PilotGuideOutputFormat::Text,
                ..
            })
        }
    ));
}

#[test]
fn cli_parses_pilot_evidence_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot-evidence",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PilotEvidence(PilotEvidenceArgs {
            format: PilotGuideOutputFormat::Text,
            ..
        })
    ));
}
