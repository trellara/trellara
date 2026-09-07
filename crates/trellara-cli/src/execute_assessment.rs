use crate::*;

pub(crate) fn quickstart_command(args: QuickstartArgs) -> Result<String> {
    if args.check {
        render_quickstart_readiness_summary(&quickstart_readiness(&args)?, args.format)
    } else {
        render_quickstart_summary(&QuickstartSummary::from_args(&args), args.format)
    }
}

pub(crate) fn mvp_check_command(args: MvpCheckArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    render_mvp_readiness_summary(
        &MvpReadinessSummary::from_config(&config, &args.config)?,
        args.format,
    )
}

pub(crate) fn evaluate_command(args: EvaluateArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    render_enterprise_evaluation_summary(
        &EnterpriseEvaluationSummary::from_config(&config, &args.config),
        args.format,
    )
}

pub(crate) fn consistency_command(args: ConsistencyContractArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    render_consistency_contract_summary(
        &ConsistencyContractSummary::from_config(&config, &args.config)?,
        args.format,
    )
}

pub(crate) fn performance_command(args: PerformanceEnvelopeArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    render_performance_envelope_summary(
        &PerformanceEnvelopeSummary::from_config(&config, &args.config),
        args.format,
    )
}

pub(crate) fn identity_audit_command(args: IdentityAuditArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    render_identity_audit_summary(
        &IdentityAuditSummary::from_config(&config, &args.config),
        args.format,
    )
}

pub(crate) fn evidence_registry_command(args: EvidenceRegistryArgs) -> Result<String> {
    render_evidence_registry_summary(
        &EvidenceRegistrySummary::from_package(
            &args.package,
            Some(args.correctness_report.as_path()),
        )?,
        args.format,
    )
}

pub(crate) fn semantics_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    Ok(serde_json::to_string_pretty(
        &ConsumerSemanticsSummary::from_config(&config, &args.config)?,
    )?)
}

pub(crate) fn explain_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    Ok(config.explain())
}
