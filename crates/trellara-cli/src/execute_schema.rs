use std::fs;

use trellara_pg_capture::PgCapture;
use trellara_protocol::TransactionEnvelope;

use crate::*;

pub(crate) async fn execute_schema_command(command: SchemaCommand) -> Result<String> {
    match command {
        SchemaCommand::Discover(args) => schema_discover_command(args).await,
        SchemaCommand::DdlPlan(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            render_ddl_plan_summary(&DdlPlanSummary::from_args(&config, &args)?, args.format)
        }
        SchemaCommand::DdlApplyPlan(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            render_ddl_apply_plan_summary(
                &DdlApplyPlanSummary::from_args(&config, &args)?,
                args.format,
            )
        }
        SchemaCommand::DdlEnvelopePlan(args) => {
            let config = TrellaraConfig::from_path(&args.config)?;
            config.validate()?;
            let bytes = fs::read(&args.file).map_err(|source| CliError::ReadInput {
                path: args.file.display().to_string(),
                source,
            })?;
            let envelope = TransactionEnvelope::decode_checked(&bytes)?;
            render_ddl_envelope_plan_summary(
                &DdlEnvelopePlanSummary::from_envelope(&config, &envelope)?,
                args.format,
            )
        }
        SchemaCommand::DdlBarrier { command } => match *command {
            DdlBarrierCommand::Record(args) => record_ddl_barrier_command(args).await,
            DdlBarrierCommand::Ack(args) => ack_ddl_barrier_command(*args).await,
            DdlBarrierCommand::Status(args) => ddl_barrier_status_command(args).await,
        },
    }
}

pub(crate) async fn schema_discover_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    let capture = PgCapture::connect(config.to_capture_config(false)?).await?;
    let preflight = apply_preflight_contracts(&config, capture.inspect_tables().await?).await?;

    Ok(serde_json::to_string_pretty(
        &SchemaDiscoverySummary::from_preflight(&config, preflight),
    )?)
}
