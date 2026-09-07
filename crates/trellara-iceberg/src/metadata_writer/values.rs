use trellara_lake::{LakeCompletenessState, LakeEpochSourceState, LakeEpochVerificationStatus};

use crate::{IcebergIntegrationError, Result};

#[derive(Clone, Debug)]
pub(super) enum MetadataValue {
    String(Option<String>),
    Int(Option<i32>),
    Long(Option<i64>),
}

pub(super) fn string(value: &str) -> MetadataValue {
    MetadataValue::String(Some(value.to_string()))
}

pub(super) fn optional_string(value: Option<&str>) -> MetadataValue {
    MetadataValue::String(value.map(ToString::to_string))
}

pub(super) fn count(field: &'static str, value: usize) -> Result<MetadataValue> {
    i64::try_from(value)
        .map(|value| MetadataValue::Long(Some(value)))
        .map_err(|_| IcebergIntegrationError::CountOverflow { field })
}

pub(super) fn integer(field: &'static str, value: u32) -> Result<MetadataValue> {
    i32::try_from(value)
        .map(|value| MetadataValue::Int(Some(value)))
        .map_err(|_| IcebergIntegrationError::CountOverflow { field })
}

pub(super) fn checksum(value: u64) -> MetadataValue {
    MetadataValue::Long(Some(i64::from_ne_bytes(value.to_ne_bytes())))
}

pub(super) fn source_state(state: LakeEpochSourceState) -> &'static str {
    match state {
        LakeEpochSourceState::Complete => "complete",
        LakeEpochSourceState::Lagging => "lagging",
        LakeEpochSourceState::Missing => "missing",
        LakeEpochSourceState::Quarantined => "quarantined",
        LakeEpochSourceState::Reseeding => "reseeding",
    }
}

pub(super) fn completeness_state(state: LakeCompletenessState) -> &'static str {
    match state {
        LakeCompletenessState::Open => "open",
        LakeCompletenessState::Sealing => "sealing",
        LakeCompletenessState::Complete => "complete",
        LakeCompletenessState::CompleteWithGaps => "complete_with_gaps",
        LakeCompletenessState::Quarantined => "quarantined",
        LakeCompletenessState::Reseeding => "reseeding",
        LakeCompletenessState::FailedRecoverable => "failed_recoverable",
    }
}

pub(super) fn verification_status(status: LakeEpochVerificationStatus) -> &'static str {
    match status {
        LakeEpochVerificationStatus::Match => "match",
        LakeEpochVerificationStatus::Mismatch => "mismatch",
        LakeEpochVerificationStatus::Unknown => "unknown",
    }
}
