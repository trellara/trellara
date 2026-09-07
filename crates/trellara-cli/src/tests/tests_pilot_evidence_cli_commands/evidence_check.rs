use super::*;

#[test]
fn cli_parses_pilot_evidence_check_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot",
        "evidence-check",
        "--config",
        "examples/retail-fleet/local.yml",
        "--evidence-dir",
        "target/evidence",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Pilot {
            command: PilotCommand::EvidenceCheck(PilotEvidenceCheckArgs {
                evidence_dir,
                format: PilotGuideOutputFormat::Text,
                ..
            })
        } if evidence_dir.as_path() == Path::new("target/evidence")
    ));
}

#[test]
fn cli_parses_pilot_evidence_check_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot-evidence-check",
        "--config",
        "examples/retail-fleet/local.yml",
        "--evidence-dir",
        "target/evidence",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PilotEvidenceCheck(PilotEvidenceCheckArgs {
            evidence_dir,
            format: PilotGuideOutputFormat::Text,
            ..
        }) if evidence_dir.as_path() == Path::new("target/evidence")
    ));
}
