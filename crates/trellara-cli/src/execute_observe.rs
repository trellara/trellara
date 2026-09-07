use crate::*;

pub(crate) async fn status_command(args: StatusArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let status = flow_status(&config).await?;

    render_status_view(status, args.view, args.format, &args.config)
}

pub(crate) async fn correctness_report_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let status = flow_status(&config).await?;

    Ok(serde_json::to_string_pretty(
        &CorrectnessReportSummary::from_status(status),
    )?)
}

pub(crate) async fn alerts_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let status = flow_status(&config).await?;

    Ok(serde_json::to_string_pretty(
        &FlowAlertsSummary::from_status(status),
    )?)
}

pub(crate) async fn dashboard_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let status = flow_status(&config).await?;

    Ok(serde_json::to_string_pretty(
        &DashboardSummary::from_status(status),
    )?)
}

pub(crate) async fn metrics_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let status = flow_status(&config).await?;

    Ok(render_prometheus_metrics(status))
}

pub(crate) async fn diagnostics_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let status = flow_status(&config).await?;

    Ok(serde_json::to_string_pretty(
        &DiagnosticsBundleSummary::from_status(status, &args.config),
    )?)
}

pub(crate) async fn execute_repair_command(command: RepairCommand) -> Result<String> {
    match command {
        RepairCommand::Plan(args) => repair_plan_command(args).await,
    }
}

pub(crate) async fn repair_plan_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let status = flow_status(&config).await?;

    Ok(serde_json::to_string_pretty(
        &RepairPlanSummary::from_status(status, &args.config),
    )?)
}
