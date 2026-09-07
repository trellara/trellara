use super::RawCdcMetadataInput;
use crate::LakeError;

#[path = "metadata_validation/fields.rs"]
mod fields;
#[path = "metadata_validation/rollups.rs"]
mod rollups;
#[path = "metadata_validation/rows.rs"]
mod rows;

pub(super) fn validate_metadata_rows(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    rows::validate(input)
}

pub(super) fn validate_metadata_rollups(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    rollups::validate(input)
}
