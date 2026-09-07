use crate::{
    lsn::{format_lsn, lsn_shape_is_valid},
    parse_lsn, CheckpointError, Result,
};

pub(crate) fn validate_non_empty(context: &str, field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "{context} {field} must not be empty"
        )));
    }
    Ok(())
}

pub(crate) fn validate_no_surrounding_whitespace(
    context: &str,
    field: &'static str,
    value: &str,
) -> Result<()> {
    if value.trim() != value {
        return Err(CheckpointError::Store(format!(
            "{context} {field} must not contain surrounding whitespace"
        )));
    }
    Ok(())
}

pub(crate) fn validate_identity_field(
    context: &str,
    field: &'static str,
    value: &str,
) -> Result<()> {
    validate_non_empty(context, field, value)?;
    validate_no_surrounding_whitespace(context, field, value)
}

pub(crate) fn validate_non_zero_lsn(field: &'static str, lsn: &str) -> Result<()> {
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(CheckpointError::Store(format!(
            "DDL barrier {field} {lsn} must be a non-zero PostgreSQL LSN like 0/16B9000"
        )));
    }
    Ok(())
}

pub(crate) fn canonical_lsn(lsn: &str) -> String {
    format_lsn(parse_lsn(lsn))
}
