use std::collections::HashSet;

use crate::{
    ddl_barrier_policy_validation::validate_propagation_policy_evidence,
    ddl_barrier_validation_fields::{
        canonical_lsn, validate_identity_field, validate_no_surrounding_whitespace,
        validate_non_empty, validate_non_zero_lsn,
    },
    CheckpointError, DdlBarrier, Result, DDL_BARRIER_CDC_TRANSACTION_BOUNDARY,
};

pub(crate) fn normalize_ddl_barrier(mut barrier: DdlBarrier) -> Result<DdlBarrier> {
    validate_ddl_barrier(&barrier)?;
    barrier.barrier_lsn = canonical_lsn(&barrier.barrier_lsn);
    Ok(barrier)
}

pub(crate) fn validate_ddl_barrier(barrier: &DdlBarrier) -> Result<()> {
    validate_identity_field("DDL barrier", "source_id", &barrier.source_id)?;
    validate_identity_field("DDL barrier", "database_id", &barrier.database_id)?;
    validate_identity_field("DDL barrier", "dataset_id", &barrier.dataset_id)?;
    validate_identity_field("DDL barrier", "id", &barrier.barrier_id)?;
    if barrier.barrier_lsn.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} cannot be recorded without a barrier LSN",
            barrier.barrier_id
        )));
    }
    validate_non_zero_lsn("barrier_lsn", &barrier.barrier_lsn)?;
    if barrier.schema_version.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} cannot be recorded without a schema version",
            barrier.barrier_id
        )));
    }
    validate_no_surrounding_whitespace(
        &format!("DDL barrier {}", barrier.barrier_id),
        "schema_version",
        &barrier.schema_version,
    )?;
    validate_non_empty(
        "DDL barrier",
        "cdc_transaction_boundary",
        &barrier.cdc_transaction_boundary,
    )?;
    validate_no_surrounding_whitespace(
        &format!("DDL barrier {}", barrier.barrier_id),
        "cdc_transaction_boundary",
        &barrier.cdc_transaction_boundary,
    )?;
    validate_cdc_transaction_boundary(barrier)?;
    if barrier.required_sinks.is_empty()
        || barrier
            .required_sinks
            .iter()
            .any(|sink| sink.trim().is_empty())
    {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} requires at least one non-empty sink acknowledgement",
            barrier.barrier_id
        )));
    }
    if barrier
        .required_sinks
        .iter()
        .any(|sink| sink.trim() != sink)
    {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} required sink names must not contain surrounding whitespace",
            barrier.barrier_id
        )));
    }
    let unique_sinks = barrier.required_sinks.iter().collect::<HashSet<_>>();
    if unique_sinks.len() != barrier.required_sinks.len() {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} has duplicate required sinks",
            barrier.barrier_id
        )));
    }
    if barrier.requires_global_partition_pause
        && !barrier
            .required_sinks
            .iter()
            .any(|sink| sink == "partition_visibility")
    {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} requires partition_visibility acknowledgement when global partition pause is enabled",
            barrier.barrier_id
        )));
    }

    Ok(())
}

fn validate_cdc_transaction_boundary(barrier: &DdlBarrier) -> Result<()> {
    if !barrier
        .cdc_transaction_boundary
        .starts_with(DDL_BARRIER_CDC_TRANSACTION_BOUNDARY)
    {
        return Err(invalid_cdc_boundary(
            barrier,
            "must start with the canonical source commit LSN DDL barrier",
        ));
    }
    if !barrier.cdc_transaction_boundary.contains("post-DDL DML") {
        return Err(invalid_cdc_boundary(
            barrier,
            "must describe how post-DDL DML is held until release",
        ));
    }
    validate_propagation_policy_evidence(barrier)
}

fn invalid_cdc_boundary(barrier: &DdlBarrier, reason: &str) -> CheckpointError {
    CheckpointError::Store(format!(
        "DDL barrier {} cdc_transaction_boundary is invalid: {reason}",
        barrier.barrier_id
    ))
}
