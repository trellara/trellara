use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

use crate::LakeError;

pub(crate) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), LakeError> {
    if value.trim().is_empty() {
        return Err(LakeError::MissingDdlAckField { field });
    }
    Ok(())
}

pub(crate) fn validate_clean_field(field: &'static str, value: &str) -> Result<(), LakeError> {
    validate_non_empty(field, value)?;
    validate_no_surrounding_whitespace(field, value)
}

pub(crate) fn validate_no_surrounding_whitespace(
    field: &'static str,
    value: &str,
) -> Result<(), LakeError> {
    if value.trim() != value {
        return Err(LakeError::InvalidDdlAckField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn validate_ack_lsn(ack_lsn: &str) -> Result<(), LakeError> {
    validate_non_empty("ack_lsn", ack_lsn)?;
    validate_no_surrounding_whitespace("ack_lsn", ack_lsn)?;
    if !lsn_shape_is_valid(ack_lsn) || parse_lsn(ack_lsn) == 0 {
        return Err(LakeError::InvalidDdlAckLsn {
            ack_lsn: ack_lsn.to_string(),
        });
    }
    Ok(())
}

pub(crate) fn validate_sha256_digest(field: &'static str, value: &str) -> Result<(), LakeError> {
    validate_clean_field(field, value)?;
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }
    Err(LakeError::InvalidDdlAckField {
        field,
        reason: "must be a 64-character SHA-256 hex digest".to_string(),
    })
}
