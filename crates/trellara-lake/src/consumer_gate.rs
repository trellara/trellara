use serde::{Deserialize, Serialize};

use crate::consumer_gate_consistency::{
    validate_epoch_consistency, validate_epoch_metadata_identity, validate_verification_counts,
};
use crate::{
    LakeCompletenessState, LakeEpochVerificationStatus, LakeError, LakeRawCdcEpochRow,
    LakeRawCdcEpochVerificationRow,
};

pub const LAKE_EPOCH_CONSUMER_GATE_CONTRACT: &str =
    "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LakeEpochConsumerOptions {
    pub accept_complete_with_gaps: bool,
}

impl LakeEpochConsumerOptions {
    pub fn strict() -> Self {
        Self {
            accept_complete_with_gaps: false,
        }
    }

    pub fn accepting_complete_with_gaps() -> Self {
        Self {
            accept_complete_with_gaps: true,
        }
    }
}

impl Default for LakeEpochConsumerOptions {
    fn default() -> Self {
        Self::strict()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpochConsumerDecision {
    pub contract: &'static str,
    pub epoch_id: String,
    pub dataset_id: String,
    pub state: LakeCompletenessState,
    pub verification_status: LakeEpochVerificationStatus,
    pub required_source_count: usize,
    pub complete_source_count: usize,
    pub missing_source_count: usize,
    pub quarantined_source_count: usize,
    pub manifest_digest: String,
    pub accepted_gaps: bool,
    pub release_gate: String,
}

pub fn ensure_lake_epoch_consumable(
    epoch: &LakeRawCdcEpochRow,
    verification: &LakeRawCdcEpochVerificationRow,
    options: LakeEpochConsumerOptions,
) -> Result<LakeEpochConsumerDecision, LakeError> {
    validate_epoch_metadata_identity(epoch, verification)?;
    validate_epoch_verification_boundary(epoch, verification)?;
    validate_epoch_consistency(epoch)?;
    validate_verification_counts(epoch, verification)?;
    validate_epoch_state(epoch, options)?;
    validate_epoch_verification_match(epoch, verification)?;
    Ok(consumer_decision(epoch, verification, options))
}

fn validate_epoch_verification_boundary(
    epoch: &LakeRawCdcEpochRow,
    verification: &LakeRawCdcEpochVerificationRow,
) -> Result<(), LakeError> {
    if epoch.epoch_id == verification.epoch_id {
        return Ok(());
    }
    Err(LakeError::EpochVerificationBoundaryMismatch {
        epoch_id: epoch.epoch_id.clone(),
        verification_epoch_id: verification.epoch_id.clone(),
    })
}

fn validate_epoch_verification_match(
    epoch: &LakeRawCdcEpochRow,
    verification: &LakeRawCdcEpochVerificationRow,
) -> Result<(), LakeError> {
    if verification.checksum_status == LakeEpochVerificationStatus::Match {
        return Ok(());
    }
    Err(LakeError::EpochVerificationNotMatched {
        epoch_id: epoch.epoch_id.clone(),
        verification_status: verification.checksum_status,
    })
}

fn validate_epoch_state(
    epoch: &LakeRawCdcEpochRow,
    options: LakeEpochConsumerOptions,
) -> Result<(), LakeError> {
    match epoch.state {
        LakeCompletenessState::Complete => Ok(()),
        LakeCompletenessState::CompleteWithGaps if options.accept_complete_with_gaps => Ok(()),
        LakeCompletenessState::CompleteWithGaps => Err(LakeError::EpochRequiresGapAcceptance {
            epoch_id: epoch.epoch_id.clone(),
        }),
        state => Err(LakeError::EpochNotConsumable {
            epoch_id: epoch.epoch_id.clone(),
            state,
        }),
    }
}

fn consumer_decision(
    epoch: &LakeRawCdcEpochRow,
    verification: &LakeRawCdcEpochVerificationRow,
    options: LakeEpochConsumerOptions,
) -> LakeEpochConsumerDecision {
    LakeEpochConsumerDecision {
        contract: LAKE_EPOCH_CONSUMER_GATE_CONTRACT,
        epoch_id: epoch.epoch_id.clone(),
        dataset_id: epoch.dataset_id.clone(),
        state: epoch.state,
        verification_status: verification.checksum_status,
        required_source_count: epoch.required_source_count,
        complete_source_count: epoch.complete_source_count,
        missing_source_count: epoch.missing_source_count,
        quarantined_source_count: epoch.quarantined_source_count,
        manifest_digest: epoch.manifest_digest.clone(),
        accepted_gaps: options.accept_complete_with_gaps
            && epoch.state == LakeCompletenessState::CompleteWithGaps,
        release_gate: "verified epoch metadata is consumable for downstream lake jobs".to_string(),
    }
}
