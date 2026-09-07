use trellara_checkpoint::{DdlBarrierLookup, DdlBarrierStore, PostgresCheckpointStore};

use crate::ddl_barrier_ack::ddl_barrier_ack_from_args;

use crate::{
    config_source_database_id, ddl_barrier_from_plan, render_ddl_barrier_summary, CliError,
    DdlBarrierAckArgs, DdlBarrierRecordArgs, DdlBarrierStatusArgs, DdlPlanArgs, DdlPlanSummary,
    DdlPlanVerdict, QuickstartOutputFormat, Result, TrellaraConfig,
};

async fn target_checkpoint_store(config: &TrellaraConfig) -> Result<PostgresCheckpointStore> {
    let target = config
        .target
        .as_ref()
        .ok_or_else(|| CliError::InvalidConfig("target.database_url is required".to_string()))?;
    Ok(PostgresCheckpointStore::connect(&target.database_url, true).await?)
}

pub(crate) async fn record_ddl_barrier_command(args: DdlBarrierRecordArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let plan_args = DdlPlanArgs {
        config: args.config.clone(),
        changes: args.changes.clone(),
        apply_mode: args.apply_mode,
        format: QuickstartOutputFormat::Json,
    };
    let plan = DdlPlanSummary::from_args(&config, &plan_args)?;
    if plan.verdict == DdlPlanVerdict::Blocked {
        return Err(CliError::InvalidConfig(
            "cannot record a DDL barrier for a blocked DDL plan".to_string(),
        ));
    }
    let barrier = ddl_barrier_from_plan(&config, &plan, &args.barrier_lsn, &args.schema_version)?;
    let barrier_id = barrier.barrier_id.clone();
    let store = target_checkpoint_store(&config).await?;
    store.record_ddl_barrier(barrier).await?;
    let lookup = ddl_barrier_lookup(&config);
    let summary = store
        .ddl_barrier_summary(&lookup, &barrier_id)
        .await?
        .ok_or_else(|| {
            CliError::InvalidConfig(format!(
                "DDL barrier {barrier_id} was recorded but could not be loaded"
            ))
        })?;

    render_ddl_barrier_summary(&summary, args.format)
}

pub(crate) async fn ack_ddl_barrier_command(args: DdlBarrierAckArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let store = target_checkpoint_store(&config).await?;
    store
        .record_ddl_barrier_ack(ddl_barrier_ack_from_args(&config, &args)?)
        .await?;
    let lookup = ddl_barrier_lookup(&config);
    let summary = store
        .ddl_barrier_summary(&lookup, &args.barrier_id)
        .await?
        .ok_or_else(|| {
            CliError::InvalidConfig(format!("DDL barrier {} is missing", args.barrier_id))
        })?;

    render_ddl_barrier_summary(&summary, args.format)
}

pub(crate) async fn ddl_barrier_status_command(args: DdlBarrierStatusArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let store = target_checkpoint_store(&config).await?;
    let lookup = ddl_barrier_lookup(&config);
    let summary = store
        .ddl_barrier_summary(&lookup, &args.barrier_id)
        .await?
        .ok_or_else(|| {
            CliError::InvalidConfig(format!("DDL barrier {} is missing", args.barrier_id))
        })?;

    render_ddl_barrier_summary(&summary, args.format)
}

fn ddl_barrier_lookup(config: &TrellaraConfig) -> DdlBarrierLookup {
    DdlBarrierLookup::new(
        &config.source.id,
        config_source_database_id(config),
        &config.dataset.id,
    )
}
