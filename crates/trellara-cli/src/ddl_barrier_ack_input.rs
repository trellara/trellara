use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

use crate::{CliError, Result};

pub(crate) fn required<T>(value: Option<T>, field: &'static str) -> Result<T> {
    value.ok_or_else(|| {
        CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --sink requires --{}",
            field.replace('_', "-")
        ))
    })
}

pub(crate) fn required_clean<'a>(value: Option<&'a str>, field: &'static str) -> Result<&'a str> {
    let value = required(value, field)?;
    if value.trim().is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{} must not be empty",
            field.replace('_', "-")
        )));
    }
    if value.trim() != value {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{} must not contain surrounding whitespace",
            field.replace('_', "-")
        )));
    }
    Ok(value)
}

pub(crate) fn validate_detail(detail: &str) -> Result<()> {
    if detail.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --detail must include evidence".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_barrier_id(barrier_id: &str) -> Result<()> {
    if barrier_id.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --barrier-id must not be empty".to_string(),
        ));
    }
    if barrier_id.trim() != barrier_id {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --barrier-id must not contain surrounding whitespace"
                .to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_sink(sink: &str) -> Result<()> {
    if sink.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --sink must not be empty".to_string(),
        ));
    }
    if sink.trim() != sink {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --sink must not contain surrounding whitespace".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_ack_lsn(ack_lsn: &str) -> Result<()> {
    validate_lsn_arg("ack-lsn", ack_lsn)
}

pub(crate) fn validate_barrier_lsn(barrier_lsn: &str) -> Result<()> {
    validate_lsn_arg("barrier-lsn", barrier_lsn)
}

fn validate_lsn_arg(flag: &str, lsn: &str) -> Result<()> {
    if lsn.trim().is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{flag} must not be empty"
        )));
    }
    if lsn.trim() != lsn {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{flag} must not contain surrounding whitespace"
        )));
    }
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{flag} {lsn} must be a non-zero PostgreSQL LSN like 0/16B9000"
        )));
    }
    Ok(())
}

pub(crate) fn validate_schema_version(schema_version: &str) -> Result<()> {
    if schema_version.trim().is_empty() {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --schema-version must not be empty".to_string(),
        ));
    }
    if schema_version.trim() != schema_version {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --schema-version must not contain surrounding whitespace"
                .to_string(),
        ));
    }
    Ok(())
}
