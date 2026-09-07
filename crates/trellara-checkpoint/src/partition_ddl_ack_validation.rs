use crate::partition_ddl_ack_watermark::validate_watermark_summary_consistency;
use crate::{
    lsn::lsn_shape_is_valid, parse_lsn, CheckpointError, PartitionVisibilityDdlAckRequest, Result,
};

pub(crate) fn validate_request(request: &PartitionVisibilityDdlAckRequest) -> Result<()> {
    validate_non_empty("source_id", &request.source_id)?;
    validate_no_surrounding_whitespace("source_id", &request.source_id)?;
    validate_non_empty("database_id", &request.database_id)?;
    validate_no_surrounding_whitespace("database_id", &request.database_id)?;
    validate_non_empty("dataset_id", &request.dataset_id)?;
    validate_no_surrounding_whitespace("dataset_id", &request.dataset_id)?;
    validate_non_empty("barrier_id", &request.barrier_id)?;
    validate_no_surrounding_whitespace("barrier_id", &request.barrier_id)?;
    validate_lsn("barrier_lsn", &request.barrier_lsn)?;
    validate_non_empty("schema_version", &request.schema_version)?;
    validate_no_surrounding_whitespace("schema_version", &request.schema_version)?;
    validate_watermark_identity(request)?;
    validate_watermark_summary_consistency(&request.watermarks)?;
    validate_complete_partition_set(request)?;
    validate_global_watermarks_reach_barrier(request)
}

fn validate_watermark_identity(request: &PartitionVisibilityDdlAckRequest) -> Result<()> {
    if request.source_id != request.watermarks.source_id
        || request.dataset_id != request.watermarks.dataset_id
    {
        return Err(store_error(
            "partition visibility DDL ack watermarks must match source_id and dataset_id",
        ));
    }
    Ok(())
}

fn validate_complete_partition_set(request: &PartitionVisibilityDdlAckRequest) -> Result<()> {
    if !request.watermarks.complete_partition_set {
        return Err(store_error(format!(
            "partition visibility DDL ack is missing partitions {:?}",
            request.watermarks.missing_partitions
        )));
    }
    Ok(())
}

fn validate_global_watermarks_reach_barrier(
    request: &PartitionVisibilityDdlAckRequest,
) -> Result<()> {
    let ack_lsn = request
        .watermarks
        .global_applied_lsn
        .as_deref()
        .ok_or_else(|| store_error("partition visibility DDL ack is missing global_applied_lsn"))?;
    validate_lsn("global_applied_lsn", ack_lsn)?;
    let durable_lsn = request
        .watermarks
        .global_durable_lsn
        .as_deref()
        .ok_or_else(|| store_error("partition visibility DDL ack is missing global_durable_lsn"))?;
    validate_lsn("global_durable_lsn", durable_lsn)?;
    if parse_lsn(durable_lsn) < parse_lsn(&request.barrier_lsn) {
        return Err(store_error(format!(
            "partition visibility global_durable_lsn {durable_lsn} is before barrier_lsn {}",
            request.barrier_lsn
        )));
    }
    if parse_lsn(ack_lsn) < parse_lsn(&request.barrier_lsn) {
        return Err(store_error(format!(
            "partition visibility global_applied_lsn {ack_lsn} is before barrier_lsn {}",
            request.barrier_lsn
        )));
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(store_error(format!(
            "partition visibility DDL ack is missing {field}"
        )));
    }
    Ok(())
}

fn validate_no_surrounding_whitespace(field: &'static str, value: &str) -> Result<()> {
    if value.trim() != value {
        return Err(store_error(format!(
            "partition visibility DDL ack {field} must not contain surrounding whitespace"
        )));
    }
    Ok(())
}

fn validate_lsn(field: &'static str, lsn: &str) -> Result<()> {
    validate_non_empty(field, lsn)?;
    validate_no_surrounding_whitespace(field, lsn)?;
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(store_error(format!(
            "partition visibility DDL ack {field} {lsn} must be a non-zero PostgreSQL LSN like 0/16B9000"
        )));
    }
    Ok(())
}

fn store_error(message: impl Into<String>) -> CheckpointError {
    CheckpointError::Store(message.into())
}
