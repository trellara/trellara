use std::fs;

use trellara_protocol::TransactionEnvelope;

use crate::lake_fanin_run_types::LakeFaninRunSummary;
use crate::lake_plan_tables::lake_materialization_config;
use crate::lake_writer_config::raw_cdc_writer_config;
use crate::*;

pub(crate) fn execute_lake_command(command: LakeCommand) -> Result<String> {
    match command {
        LakeCommand::Plan(args) => render_lake_plan_command(args),
        LakeCommand::Ddl(args) => render_lake_ddl_command(args),
        LakeCommand::WriterPlan(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            let envelopes = read_lake_writer_envelopes(&args)?;
            let epoch_id = resolve_lake_epoch_id(&args.epoch_id, &config, &envelopes)?;
            let plan = trellara_lake::plan_raw_cdc_epoch_writes(
                &raw_cdc_writer_config(&config, &epoch_id, args.source_bucket_count),
                &lake_materialization_config(&config),
                &envelopes,
            )?;
            render_lake_writer_plan_summary(&plan, args.format)
        }
        LakeCommand::SparkTemplate { command } => render_lake_spark_template_command(command),
        LakeCommand::Inspect(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            let bytes = fs::read(&args.file).map_err(|source| CliError::ReadInput {
                path: args.file.display().to_string(),
                source,
            })?;
            let envelope = TransactionEnvelope::decode_checked(&bytes)?;
            let plan =
                trellara_lake::plan_commit(&envelope, &lake_materialization_config(&config))?;

            Ok(serde_json::to_string_pretty(&plan)?)
        }
        LakeCommand::Epoch(args) => render_lake_epoch_command(args),
        LakeCommand::Fanin {
            command: LakeFaninCommand::Plan(args),
        } => render_lake_plan_command(args),
        LakeCommand::Fanin {
            command: LakeFaninCommand::Ddl(args),
        } => render_lake_ddl_command(args),
        LakeCommand::Fanin {
            command: LakeFaninCommand::EpochSpec(args),
        } => render_lake_epoch_command(args),
        LakeCommand::Fanin {
            command: LakeFaninCommand::Run(args),
        } => render_lake_fanin_run_command(args),
        LakeCommand::Fanin {
            command: LakeFaninCommand::Verify(args),
        } => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            render_lake_fanin_verify_summary(
                &LakeFaninVerifySummary::from_args(&config, &args)?,
                args.format,
            )
        }
        LakeCommand::Fanin {
            command: LakeFaninCommand::Completeness(args),
        } => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            render_lake_fanin_completeness_summary(
                &LakeFaninCompletenessSummary::from_args(&config, &args)?,
                args.format,
            )
        }
    }
}

fn render_lake_spark_template_command(command: LakeSparkTemplateCommand) -> Result<String> {
    let (args, template_kind) = match command {
        LakeSparkTemplateCommand::CurrentState(args) => (args, LakeSparkTemplateKind::CurrentState),
        LakeSparkTemplateCommand::Scd2(args) => (args, LakeSparkTemplateKind::Scd2),
        LakeSparkTemplateCommand::Maintenance(args) => (args, LakeSparkTemplateKind::Maintenance),
        LakeSparkTemplateCommand::Dashboard(args) => (args, LakeSparkTemplateKind::Dashboard),
    };
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    render_lake_spark_template_summary(
        &LakeSparkTemplateSummary::from_args(&config, &args, template_kind)?,
        args.format,
    )
}

fn render_lake_plan_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    Ok(serde_json::to_string_pretty(
        &LakePlanSummary::from_config(&config),
    )?)
}

fn render_lake_ddl_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    Ok(serde_json::to_string_pretty(&LakeDdlSummary::from_config(
        &config,
    ))?)
}

fn render_lake_epoch_command(args: LakeEpochArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    render_lake_epoch_summary(&LakeEpochSummary::from_config(&config, &args), args.format)
}

fn render_lake_fanin_run_command(args: LakeFaninRunArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let envelopes = read_lake_fanin_run_envelopes(&args)?;
    let epoch_id = resolve_lake_epoch_id(&args.epoch_id, &config, &envelopes)?;
    let plan = trellara_lake::plan_raw_cdc_epoch_writes(
        &raw_cdc_writer_config(&config, &epoch_id, args.source_bucket_count),
        &lake_materialization_config(&config),
        &envelopes,
    )?;
    render_lake_fanin_run_summary(
        &LakeFaninRunSummary::from_writer_plan(&config, plan),
        args.format,
    )
}
