use crate::{lsn::lsn_shape_is_valid, parse_lsn, CheckpointError, Result};

pub(crate) fn validate_optional_nonzero_lsn(
    context: &str,
    field: &str,
    lsn: Option<&str>,
) -> Result<()> {
    let Some(lsn) = lsn else {
        return Ok(());
    };
    if lsn.trim().is_empty() {
        return Ok(());
    }
    validate_nonzero_lsn(context, field, lsn)
}

pub(crate) fn validate_required_nonzero_lsn(context: &str, field: &str, lsn: &str) -> Result<()> {
    if lsn.trim().is_empty() {
        return Ok(());
    }
    validate_nonzero_lsn(context, field, lsn)
}

fn validate_nonzero_lsn(context: &str, field: &str, lsn: &str) -> Result<()> {
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(CheckpointError::Store(format!(
            "{context} {field} {lsn:?} must be a non-zero PostgreSQL LSN like 0/16B8000"
        )));
    }
    Ok(())
}
