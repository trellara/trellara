use crate::*;

pub(crate) async fn run_command(args: RunArgs) -> Result<String> {
    run_configured_flow(&args).await
}

pub(crate) fn execute_dev_command(command: DevCommand) -> Result<String> {
    match command {
        DevCommand::Up(args) => dev_up_command(&args),
        DevCommand::Down(args) => dev_down_command(&args),
        DevCommand::Logs(args) => dev_logs_command(&args),
    }
}

pub(crate) async fn execute_flow_command(command: FlowCommand) -> Result<String> {
    match command {
        FlowCommand::Validate(args) => flow_validate_command(args),
        FlowCommand::Create(args) => flow_create_command(args),
        FlowCommand::Status(args) => flow_status_command(args).await,
    }
}

pub(crate) fn flow_validate_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    Ok(format!(
        "valid config_version={} environment={} source={} dataset={} mode={}",
        config.config_version,
        config.environment,
        config.source.id,
        config.dataset.id,
        config.dataset.mode
    ))
}

fn flow_create_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    Ok(serde_json::to_string_pretty(
        &FlowCreateSummary::from_config(&config, &args.config),
    )?)
}

async fn flow_status_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let summary = flow_status(&config).await?;

    Ok(serde_json::to_string_pretty(&summary)?)
}

pub(crate) fn execute_chaos_command(command: ChaosCommand) -> Result<String> {
    match command {
        ChaosCommand::Run => Ok(serde_json::to_string_pretty(&ChaosRunSummary::default())?),
        ChaosCommand::Report(args) => {
            Ok(serde_json::to_string_pretty(&write_chaos_report(&args)?)?)
        }
    }
}
