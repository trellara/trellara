use trellara_lake::LakeCompletenessState;

pub(crate) fn lake_epoch_customer_decision(state: LakeCompletenessState) -> &'static str {
    match state {
        LakeCompletenessState::Complete => "safe_for_default_spark_consumption",
        LakeCompletenessState::CompleteWithGaps => {
            "requires_explicit_gap_acceptance_before_spark_consumption"
        }
        LakeCompletenessState::Quarantined => "blocked_until_quarantine_is_resolved",
        LakeCompletenessState::Open | LakeCompletenessState::Sealing => {
            "not_visible_until_epoch_seals"
        }
        LakeCompletenessState::Reseeding | LakeCompletenessState::FailedRecoverable => {
            "requires_recovery_before_consumption"
        }
    }
}

pub(crate) fn lake_epoch_recommended_next_steps(state: LakeCompletenessState) -> Vec<String> {
    match state {
        LakeCompletenessState::Complete => vec![
            "run Spark current-state and SCD2 templates for this epoch".to_string(),
            "publish epoch metadata with verification_status=match".to_string(),
        ],
        LakeCompletenessState::CompleteWithGaps => vec![
            "review _trellara_epoch_sources before accepting this epoch".to_string(),
            "set accept_complete_with_gaps=true only for jobs that can tolerate missing stores"
                .to_string(),
            "wait for late stores and recompute the epoch when they return".to_string(),
        ],
        LakeCompletenessState::Quarantined => vec![
            "inspect _trellara_quarantine and resolve conflicting source evidence".to_string(),
            "do not expose derived current-state or SCD2 tables for this epoch".to_string(),
            "rerun lake epoch verification after repair or reseed".to_string(),
        ],
        LakeCompletenessState::Open | LakeCompletenessState::Sealing => {
            vec!["wait for the epoch visibility boundary before publishing metadata".to_string()]
        }
        LakeCompletenessState::Reseeding | LakeCompletenessState::FailedRecoverable => {
            vec!["complete recovery before publishing this epoch to consumers".to_string()]
        }
    }
}
