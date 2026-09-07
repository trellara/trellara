use crate::*;

pub(crate) fn execute_pilot_command(command: PilotCommand) -> Result<String> {
    match command {
        PilotCommand::Guide(args) => pilot_guide_command(args),
        PilotCommand::Scorecard(args) => pilot_scorecard_command(args),
        PilotCommand::Evidence(args) => pilot_evidence_command(args),
        PilotCommand::EvidenceCheck(args) => pilot_evidence_check_command(args),
        PilotCommand::EvidenceTemplate(args) => pilot_evidence_template_command(args),
        PilotCommand::Package(args) => pilot_package_command(args),
    }
}

pub(crate) fn pilot_guide_command(args: PilotGuideArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;

    render_pilot_guide_summary(
        &PilotGuideSummary::from_config(&config, &args.config),
        args.format,
    )
}

pub(crate) fn pilot_scorecard_command(args: PilotScorecardArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;

    render_pilot_scorecard_summary(
        &PilotScorecardSummary::from_config(&config, &args.config),
        args.format,
    )
}

pub(crate) fn pilot_evidence_command(args: PilotEvidenceArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;

    render_pilot_evidence_summary(
        &PilotExecutiveEvidenceSummary::from_config(&config, &args.config),
        args.format,
    )
}

pub(crate) fn pilot_evidence_check_command(args: PilotEvidenceCheckArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;

    render_pilot_evidence_check_summary(
        &PilotLiveEvidenceCheckSummary::from_config(&config, &args.config, &args.evidence_dir),
        args.format,
    )
}

pub(crate) fn pilot_evidence_template_command(args: PilotEvidenceTemplateArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;

    render_pilot_evidence_template_summary(
        &write_pilot_evidence_template(&config, &args.config, &args.output)?,
        args.format,
    )
}

pub(crate) fn pilot_package_command(args: PilotPackageArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;

    Ok(serde_json::to_string_pretty(&write_pilot_package(
        &config,
        &args.config,
        &args.output,
        &args.correctness_report,
    )?)?)
}
