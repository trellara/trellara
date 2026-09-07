use crate::{
    ddl_barrier_validation_fields::{
        canonical_lsn, validate_identity_field, validate_no_surrounding_whitespace,
        validate_non_zero_lsn,
    },
    CheckpointError, DdlBarrierAck, Result,
};

pub(crate) fn normalize_ddl_barrier_ack(mut ack: DdlBarrierAck) -> Result<DdlBarrierAck> {
    validate_ddl_barrier_ack(&ack)?;
    ack.ack_lsn = canonical_lsn(&ack.ack_lsn);
    Ok(ack)
}

pub(crate) fn validate_ddl_barrier_ack(ack: &DdlBarrierAck) -> Result<()> {
    validate_identity_field("DDL barrier ack", "source_id", &ack.source_id)?;
    validate_identity_field("DDL barrier ack", "database_id", &ack.database_id)?;
    validate_identity_field("DDL barrier ack", "dataset_id", &ack.dataset_id)?;
    validate_identity_field("DDL barrier ack", "barrier_id", &ack.barrier_id)?;
    validate_ack_sink(ack)?;
    validate_ack_lsn(ack)?;
    validate_ack_schema_version(ack)?;
    validate_ack_detail(ack)
}

fn validate_ack_sink(ack: &DdlBarrierAck) -> Result<()> {
    if ack.sink.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} ack sink must not be empty",
            ack.barrier_id
        )));
    }
    if ack.sink.trim() != ack.sink {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} ack sink must not contain surrounding whitespace",
            ack.barrier_id
        )));
    }
    Ok(())
}

fn validate_ack_lsn(ack: &DdlBarrierAck) -> Result<()> {
    if ack.ack_lsn.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} ack for {} cannot be recorded without an ACK LSN",
            ack.barrier_id, ack.sink
        )));
    }
    validate_non_zero_lsn("ack_lsn", &ack.ack_lsn)
}

fn validate_ack_schema_version(ack: &DdlBarrierAck) -> Result<()> {
    if ack.schema_version.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} ack for {} cannot be recorded without a schema version",
            ack.barrier_id, ack.sink
        )));
    }
    validate_no_surrounding_whitespace(
        &format!("DDL barrier {} ack for {}", ack.barrier_id, ack.sink),
        "schema_version",
        &ack.schema_version,
    )
}

fn validate_ack_detail(ack: &DdlBarrierAck) -> Result<()> {
    if ack.detail.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} ack for {} cannot be recorded without detail evidence",
            ack.barrier_id, ack.sink
        )));
    }
    if ack.detail.trim() != ack.detail {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} ack for {} detail evidence must not contain surrounding whitespace",
            ack.barrier_id, ack.sink
        )));
    }
    Ok(())
}
