use crate::*;

pub(crate) fn partition_local_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;

    Ok(serde_json::to_string_pretty(
        &PartitionLocalSummary::from_config(&config)?,
    )?)
}

pub(crate) async fn partition_watermarks_command(args: PartitionWatermarksArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let summary = partition_watermarks(&config).await?;

    render_partition_watermark_summary(&summary, args.format)
}

pub(crate) async fn partition_rebalance_plan_command(
    args: PartitionRebalancePlanArgs,
) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let watermarks = partition_watermarks(&config).await?;
    let plan = partition_rebalance_plan_from_watermarks(&watermarks, &args)?;

    render_partition_rebalance_plan(&plan, args.format)
}

pub(crate) async fn execute_quarantine_command(command: QuarantineCommand) -> Result<String> {
    match command {
        QuarantineCommand::List(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            let summary = list_quarantine(&config, args.limit).await?;

            Ok(serde_json::to_string_pretty(&summary)?)
        }
        QuarantineCommand::Clear(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            let summary = clear_quarantine(&config, args.transaction_id, args.commit_lsn).await?;

            Ok(serde_json::to_string_pretty(&summary)?)
        }
        QuarantineCommand::ReplayReady(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            let summary =
                prepare_quarantine_replay(&config, args.transaction_id, args.commit_lsn).await?;

            Ok(serde_json::to_string_pretty(&summary)?)
        }
    }
}
