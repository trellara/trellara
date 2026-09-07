use crate::{LakeCompletenessState, LakeError, LakeRawCdcEpochRow};

pub(crate) fn validate_epoch_completeness_state(
    epoch: &LakeRawCdcEpochRow,
) -> Result<(), LakeError> {
    match epoch.state {
        LakeCompletenessState::Complete => validate_complete_epoch(epoch),
        LakeCompletenessState::CompleteWithGaps => validate_complete_with_gaps_epoch(epoch),
        LakeCompletenessState::Quarantined => validate_quarantined_epoch(epoch),
        _ => Ok(()),
    }
}

fn validate_complete_epoch(epoch: &LakeRawCdcEpochRow) -> Result<(), LakeError> {
    if epoch.required_source_count > 0
        && epoch.missing_source_count == 0
        && epoch.quarantined_source_count == 0
        && epoch.complete_source_count == epoch.required_source_count
    {
        Ok(())
    } else {
        Err(completeness_state_mismatch(epoch))
    }
}

fn validate_complete_with_gaps_epoch(epoch: &LakeRawCdcEpochRow) -> Result<(), LakeError> {
    if epoch.policy != "publish_with_gaps" {
        return Err(LakeError::InvalidRawCdcEpochMetadataField {
            field: "epoch.policy",
            reason: "complete_with_gaps epochs must be published with publish_with_gaps policy"
                .to_string(),
        });
    }
    if epoch.missing_source_count > 0 && epoch.quarantined_source_count == 0 {
        Ok(())
    } else {
        Err(completeness_state_mismatch(epoch))
    }
}

fn validate_quarantined_epoch(epoch: &LakeRawCdcEpochRow) -> Result<(), LakeError> {
    if epoch.quarantined_source_count > 0 {
        Ok(())
    } else {
        Err(completeness_state_mismatch(epoch))
    }
}

fn completeness_state_mismatch(epoch: &LakeRawCdcEpochRow) -> LakeError {
    LakeError::EpochCompletenessStateMismatch {
        epoch_id: epoch.epoch_id.clone(),
        state: epoch.state,
        missing_source_count: epoch.missing_source_count,
        quarantined_source_count: epoch.quarantined_source_count,
    }
}
