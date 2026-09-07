use super::*;

#[test]
fn cli_parses_pilot_evidence_template_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot",
        "evidence-template",
        "--config",
        "examples/retail-fleet/local.yml",
        "--output",
        "target/evidence",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Pilot {
            command: PilotCommand::EvidenceTemplate(PilotEvidenceTemplateArgs {
                output,
                format: PilotGuideOutputFormat::Text,
                ..
            })
        } if output.as_path() == Path::new("target/evidence")
    ));
}

#[test]
fn cli_parses_pilot_evidence_template_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot-evidence-template",
        "--config",
        "examples/retail-fleet/local.yml",
        "--output",
        "target/evidence",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PilotEvidenceTemplate(PilotEvidenceTemplateArgs {
            output,
            format: PilotGuideOutputFormat::Text,
            ..
        }) if output.as_path() == Path::new("target/evidence")
    ));
}
