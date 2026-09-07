use super::*;

#[test]
fn complete_with_gaps_requires_gap_policy_metadata() {
    let mut epoch = policy_epoch_row(LakeCompletenessState::CompleteWithGaps);
    epoch.policy = "wait_all_required".to_string();
    epoch.complete_source_count = 0;
    epoch.missing_source_count = 1;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &policy_verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )
    .expect_err("gap state with strict policy rejected");

    assert!(matches!(
        err,
        LakeError::InvalidRawCdcEpochMetadataField {
            field: "epoch.policy",
            reason,
        } if reason.contains("publish_with_gaps")
    ));
}

#[test]
fn complete_with_gaps_policy_metadata_still_requires_consumer_acceptance() {
    let mut epoch = policy_epoch_row(LakeCompletenessState::CompleteWithGaps);
    epoch.policy = "publish_with_gaps".to_string();
    epoch.complete_source_count = 0;
    epoch.missing_source_count = 1;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &policy_verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("consumer acceptance still required");

    assert!(matches!(
        err,
        LakeError::EpochRequiresGapAcceptance { epoch_id } if epoch_id == "epoch-policy"
    ));
}

#[test]
fn complete_with_gaps_policy_metadata_releases_with_explicit_acceptance() {
    let mut epoch = policy_epoch_row(LakeCompletenessState::CompleteWithGaps);
    epoch.policy = "publish_with_gaps".to_string();
    epoch.complete_source_count = 0;
    epoch.missing_source_count = 1;

    let decision = ensure_lake_epoch_consumable(
        &epoch,
        &policy_verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )
    .expect("gap policy accepted");

    assert!(decision.accepted_gaps);
    assert_eq!(decision.state, LakeCompletenessState::CompleteWithGaps);
}

fn policy_epoch_row(state: LakeCompletenessState) -> LakeRawCdcEpochRow {
    LakeRawCdcEpochRow {
        epoch_id: "epoch-policy".to_string(),
        dataset_id: "retail".to_string(),
        state,
        policy: "wait_all_required".to_string(),
        opened_at: "open".to_string(),
        sealed_at: "sealed".to_string(),
        required_source_count: 1,
        complete_source_count: 1,
        missing_source_count: 0,
        quarantined_source_count: 0,
        transaction_count: 1,
        change_count: 1,
        checksum_rollup: 42,
        manifest_digest: "a".repeat(64),
        iceberg_snapshot_id: None,
    }
}

fn policy_verification_row(status: LakeEpochVerificationStatus) -> LakeRawCdcEpochVerificationRow {
    LakeRawCdcEpochVerificationRow {
        epoch_id: "epoch-policy".to_string(),
        verification_id: "verify:retail:epoch-policy:000000000000002a".to_string(),
        input_transaction_count: 1,
        input_change_count: 1,
        lake_transaction_count: 1,
        lake_change_count: 1,
        checksum_status: status,
        completed_at: "done".to_string(),
    }
}
