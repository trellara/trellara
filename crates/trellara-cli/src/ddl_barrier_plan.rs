use trellara_apply_postgres::TargetDdlBarrierRequirements;
use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn, DdlBarrier};

use crate::{CliError, DdlPlanSummary, Result, TrellaraConfig};

pub(crate) fn ddl_barrier_from_plan(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    barrier_lsn: &str,
    schema_version: &str,
) -> Result<DdlBarrier> {
    validate_barrier_lsn(barrier_lsn)?;
    validate_schema_version(schema_version)?;
    let requirements = ddl_barrier_requirements_from_plan(plan);
    Ok(DdlBarrier {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: plan.propagation.barrier_id.clone(),
        barrier_lsn: barrier_lsn.to_string(),
        schema_version: schema_version.to_string(),
        cdc_transaction_boundary: plan.propagation.cdc_transaction_boundary.clone(),
        required_sinks: requirements.required_sinks,
        requires_global_partition_pause: requirements.requires_global_partition_pause,
    })
}

pub(crate) fn config_source_database_id(config: &TrellaraConfig) -> String {
    config
        .source
        .database_id
        .clone()
        .unwrap_or_else(|| config.dataset.id.clone())
}

fn ddl_barrier_requirements_from_plan(plan: &DdlPlanSummary) -> TargetDdlBarrierRequirements {
    TargetDdlBarrierRequirements::from_required_sinks(
        plan.propagation.sinks.iter().map(|sink| sink.name.clone()),
        plan.propagation.requires_global_partition_pause,
    )
}

fn validate_barrier_lsn(barrier_lsn: &str) -> Result<()> {
    if barrier_lsn.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier record --barrier-lsn must not be empty".to_string(),
        ));
    }
    if barrier_lsn.trim() != barrier_lsn {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier record --barrier-lsn must not contain surrounding whitespace"
                .to_string(),
        ));
    }
    if !lsn_shape_is_valid(barrier_lsn) || parse_lsn(barrier_lsn) == 0 {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier record --barrier-lsn {barrier_lsn} must be a non-zero PostgreSQL LSN like 0/16B9000"
        )));
    }
    Ok(())
}

fn validate_schema_version(schema_version: &str) -> Result<()> {
    if schema_version.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier record --schema-version must not be empty".to_string(),
        ));
    }
    if schema_version.trim() != schema_version {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier record --schema-version must not contain surrounding whitespace"
                .to_string(),
        ));
    }
    Ok(())
}
