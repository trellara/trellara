use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

use crate::LakeError;

pub(super) fn validate_clean_metadata_field(
    field: &'static str,
    value: &str,
) -> Result<(), LakeError> {
    if value.trim().is_empty() {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field,
            reason: "must not be empty".to_string(),
        });
    }
    if value != value.trim() {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

pub(super) fn validate_lsn_window(
    start_field: &'static str,
    start_lsn: &str,
    end_field: &'static str,
    end_lsn: &str,
) -> Result<(), LakeError> {
    let start = validate_metadata_lsn(start_field, start_lsn)?;
    let end = validate_metadata_lsn(end_field, end_lsn)?;
    if start > end {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field: start_field,
            reason: format!("must be before or equal to {end_field} {end_lsn}"),
        });
    }
    Ok(())
}

fn validate_metadata_lsn(field: &'static str, lsn: &str) -> Result<u64, LakeError> {
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field,
            reason: format!("must be a non-zero PostgreSQL LSN like 0/16B9000, found {lsn}"),
        });
    }
    Ok(parse_lsn(lsn))
}
