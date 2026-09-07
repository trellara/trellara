pub(crate) struct LakeEpochManifestDigestInput<'a> {
    pub(crate) epoch_id: &'a str,
    pub(crate) dataset_id: &'a str,
    pub(crate) state: trellara_lake::LakeCompletenessState,
    pub(crate) straggler_policy: &'a str,
    pub(crate) required_source_count: usize,
    pub(crate) complete_source_count: usize,
    pub(crate) missing_source_count: usize,
    pub(crate) quarantined_source_count: usize,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
}

pub(crate) fn completeness_state_label(
    state: trellara_lake::LakeCompletenessState,
) -> &'static str {
    match state {
        trellara_lake::LakeCompletenessState::Open => "open",
        trellara_lake::LakeCompletenessState::Sealing => "sealing",
        trellara_lake::LakeCompletenessState::Complete => "complete",
        trellara_lake::LakeCompletenessState::CompleteWithGaps => "complete_with_gaps",
        trellara_lake::LakeCompletenessState::Quarantined => "quarantined",
        trellara_lake::LakeCompletenessState::Reseeding => "reseeding",
        trellara_lake::LakeCompletenessState::FailedRecoverable => "failed_recoverable",
    }
}
