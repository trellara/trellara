use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};
use trellara_protocol::format_lsn;

use crate::{ApplyError, Result};

pub(crate) fn validate_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ApplyError::MissingDdlField { field });
    }
    Ok(())
}

pub(crate) fn validate_no_surrounding_whitespace(field: &'static str, value: &str) -> Result<()> {
    if value.trim() != value {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn canonical_ack_lsn(ack_lsn: &str) -> Result<String> {
    canonical_lsn("ack_lsn", ack_lsn)
}

pub(crate) fn canonical_barrier_lsn(barrier_lsn: &str) -> Result<String> {
    canonical_lsn("barrier_lsn", barrier_lsn)
}

pub(crate) fn validate_ack_at_or_after_barrier(ack_lsn: &str, barrier_lsn: &str) -> Result<()> {
    if parse_lsn(ack_lsn) < parse_lsn(barrier_lsn) {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "ack_lsn",
            reason: format!("must be at or beyond barrier_lsn {barrier_lsn}"),
        });
    }
    Ok(())
}

fn canonical_lsn(field: &'static str, lsn: &str) -> Result<String> {
    validate_non_empty(field, lsn)?;
    let parsed = parse_lsn(lsn);
    if !lsn_shape_is_valid(lsn) || parsed == 0 {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field,
            reason: "must be a non-zero PostgreSQL LSN like 0/16B9000".to_string(),
        });
    }
    Ok(format_lsn(parsed))
}
