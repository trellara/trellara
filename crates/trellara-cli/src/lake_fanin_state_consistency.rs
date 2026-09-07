use crate::{lake_completeness_state_label, LakeEpochSummary};

pub(super) fn validate_epoch_state_consistency(epoch: &LakeEpochSummary) -> Result<(), String> {
    match epoch.state {
        trellara_lake::LakeCompletenessState::Complete => validate_complete_epoch(epoch),
        trellara_lake::LakeCompletenessState::CompleteWithGaps => {
            validate_complete_with_gaps_epoch(epoch)
        }
        trellara_lake::LakeCompletenessState::Quarantined => validate_quarantined_epoch(epoch),
        _ => Ok(()),
    }
}

fn validate_complete_epoch(epoch: &LakeEpochSummary) -> Result<(), String> {
    if epoch.missing_source_count == 0
        && epoch.quarantined_source_count == 0
        && epoch.complete_source_count == epoch.required_source_count
    {
        Ok(())
    } else {
        Err(state_mismatch(epoch))
    }
}

fn validate_complete_with_gaps_epoch(epoch: &LakeEpochSummary) -> Result<(), String> {
    if epoch.missing_source_count > 0 && epoch.quarantined_source_count == 0 {
        Ok(())
    } else {
        Err(state_mismatch(epoch))
    }
}

fn validate_quarantined_epoch(epoch: &LakeEpochSummary) -> Result<(), String> {
    if epoch.quarantined_source_count > 0 {
        Ok(())
    } else {
        Err(state_mismatch(epoch))
    }
}

fn state_mismatch(epoch: &LakeEpochSummary) -> String {
    format!(
        "state {} inconsistent with missing={} quarantined={}",
        lake_completeness_state_label(epoch.state),
        epoch.missing_source_count,
        epoch.quarantined_source_count
    )
}
