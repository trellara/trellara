use std::collections::BTreeMap;

use crate::epoch::{
    LakeCompletenessState, LakeEpochConfig, LakeEpochSourceState, LakeEpochVerificationStatus,
    LakeStragglerPolicy,
};
use crate::epoch_accumulator::{checked_epoch_count_add, SourceAccumulator};
use crate::LakeError;

pub(crate) fn mark_missing_sources(
    config: &LakeEpochConfig,
    sources: &mut BTreeMap<String, SourceAccumulator>,
) -> Result<(usize, usize), LakeError> {
    let mut missing_source_count = 0;
    let mut quarantined_source_count = 0;
    for (source_id, source) in sources {
        if source.transaction_count == 0 && config.required_sources.contains(source_id) {
            source.start_lsn = None;
            source.end_lsn = None;
            match config.straggler_policy {
                LakeStragglerPolicy::WaitAllRequired => {
                    source.state = LakeEpochSourceState::Lagging;
                    source.gap_reason =
                        Some("required source has not reached epoch boundary".to_string());
                }
                LakeStragglerPolicy::PublishWithGaps { .. } => {
                    source.state = LakeEpochSourceState::Missing;
                    source.gap_reason =
                        Some("required source missing from published gap epoch".to_string());
                }
                LakeStragglerPolicy::QuarantineOnGap => {
                    source.state = LakeEpochSourceState::Quarantined;
                    source.gap_reason =
                        Some("required source missing under quarantine-on-gap policy".to_string());
                }
            }
        }
        match source.state {
            LakeEpochSourceState::Missing | LakeEpochSourceState::Lagging => {
                missing_source_count =
                    checked_epoch_count_add(missing_source_count, 1, "missing_source_count")?;
            }
            LakeEpochSourceState::Quarantined => {
                quarantined_source_count = checked_epoch_count_add(
                    quarantined_source_count,
                    1,
                    "quarantined_source_count",
                )?;
            }
            LakeEpochSourceState::Complete | LakeEpochSourceState::Reseeding => {}
        }
    }
    Ok((missing_source_count, quarantined_source_count))
}

pub(crate) fn epoch_state(
    config: &LakeEpochConfig,
    missing_source_count: usize,
    quarantined_source_count: usize,
) -> LakeCompletenessState {
    if quarantined_source_count > 0 {
        LakeCompletenessState::Quarantined
    } else if missing_source_count == 0 {
        LakeCompletenessState::Complete
    } else {
        match config.straggler_policy {
            LakeStragglerPolicy::WaitAllRequired => LakeCompletenessState::Open,
            LakeStragglerPolicy::PublishWithGaps { .. } => LakeCompletenessState::CompleteWithGaps,
            LakeStragglerPolicy::QuarantineOnGap => LakeCompletenessState::Quarantined,
        }
    }
}

pub(crate) fn verification_status(state: LakeCompletenessState) -> LakeEpochVerificationStatus {
    if matches!(
        state,
        LakeCompletenessState::Complete | LakeCompletenessState::CompleteWithGaps
    ) {
        LakeEpochVerificationStatus::Match
    } else {
        LakeEpochVerificationStatus::Unknown
    }
}
