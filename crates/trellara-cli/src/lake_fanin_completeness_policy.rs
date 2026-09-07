use crate::{LakeFaninCompletenessDecision, LakeFaninVerifyStatus, LakeFaninVerifySummary};

pub(super) fn completeness_decision(
    status: LakeFaninVerifyStatus,
    spark_consumption_allowed: bool,
    state: trellara_lake::LakeCompletenessState,
    accept_complete_with_gaps: bool,
) -> LakeFaninCompletenessDecision {
    if spark_consumption_allowed && state == trellara_lake::LakeCompletenessState::CompleteWithGaps
    {
        LakeFaninCompletenessDecision::ReadyWithAcceptedGaps
    } else if spark_consumption_allowed {
        LakeFaninCompletenessDecision::Ready
    } else if status != LakeFaninVerifyStatus::Match {
        LakeFaninCompletenessDecision::BlockedVerificationMismatch
    } else if state == trellara_lake::LakeCompletenessState::CompleteWithGaps
        && !accept_complete_with_gaps
    {
        LakeFaninCompletenessDecision::BlockedNeedsGapAcceptance
    } else {
        LakeFaninCompletenessDecision::BlockedNonConsumableEpoch
    }
}

pub(super) fn recovery_path(
    decision: LakeFaninCompletenessDecision,
    verification: &LakeFaninVerifySummary,
) -> Vec<String> {
    match decision {
        LakeFaninCompletenessDecision::Ready => vec![
            "publish lake-completeness.json with the evidence bundle".to_string(),
            "run Spark-derived current-state or SCD2 jobs for consumers of this epoch".to_string(),
        ],
        LakeFaninCompletenessDecision::ReadyWithAcceptedGaps => vec![
            "publish lake-completeness.json only for consumers that explicitly accepted gaps"
                .to_string(),
            "keep missing and lagging source rows visible in the completeness dashboard"
                .to_string(),
        ],
        LakeFaninCompletenessDecision::BlockedNeedsGapAcceptance => vec![
            "review missing or lagging source rows in _trellara_epoch_sources".to_string(),
            "rerun completeness with --accept-complete-with-gaps only for tolerant consumers"
                .to_string(),
        ],
        LakeFaninCompletenessDecision::BlockedVerificationMismatch => {
            let mut steps = verification.recommended_next_steps.clone();
            steps.push(
                "regenerate stream and lake epoch artifacts before publishing Spark outputs"
                    .to_string(),
            );
            steps
        }
        LakeFaninCompletenessDecision::BlockedNonConsumableEpoch => {
            let mut steps = verification.recommended_next_steps.clone();
            steps.push(
                "inspect _trellara_quarantine, reseed state, or durable stream replay state before retrying"
                    .to_string(),
            );
            steps
        }
    }
}

pub(super) fn deferred_sink_work() -> Vec<String> {
    vec![
        "generic single-source Postgres-to-Iceberg connector positioning".to_string(),
        "native Rust current-state Iceberg upserts or deletes".to_string(),
        "merge-on-read or equality-delete authoring".to_string(),
        "automatic compaction, expiration, and orphan-file cleanup ownership".to_string(),
        "catalog-side table create or evolve execution without Trellara DDL acknowledgements"
            .to_string(),
    ]
}
