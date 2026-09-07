use super::*;

#[test]
fn strict_consumers_accept_verified_complete_epochs() {
    let decision = ensure_lake_epoch_consumable(
        &epoch_row(LakeCompletenessState::Complete),
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::strict(),
    )
    .expect("complete epoch");

    assert_eq!(decision.epoch_id, "epoch-1");
    assert_eq!(decision.contract, LAKE_EPOCH_CONSUMER_GATE_CONTRACT);
    assert_eq!(decision.required_source_count, 1);
    assert_eq!(decision.complete_source_count, 1);
    assert_eq!(decision.missing_source_count, 0);
    assert_eq!(decision.quarantined_source_count, 0);
    assert_eq!(decision.manifest_digest, "a".repeat(64));
    assert!(!decision.accepted_gaps);
}

#[test]
fn strict_consumers_refuse_complete_with_gaps() {
    let mut epoch = epoch_row(LakeCompletenessState::CompleteWithGaps);
    epoch.policy = "publish_with_gaps".to_string();
    epoch.complete_source_count = 0;
    epoch.missing_source_count = 1;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("gap acceptance required");

    assert!(matches!(
        err,
        LakeError::EpochRequiresGapAcceptance { epoch_id } if epoch_id == "epoch-1"
    ));
}

#[test]
fn gap_accepting_consumers_accept_verified_complete_with_gaps() {
    let mut epoch = epoch_row(LakeCompletenessState::CompleteWithGaps);
    epoch.policy = "publish_with_gaps".to_string();
    epoch.complete_source_count = 0;
    epoch.missing_source_count = 1;

    let decision = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )
    .expect("accepted gap epoch");

    assert!(decision.accepted_gaps);
    assert_eq!(decision.contract, LAKE_EPOCH_CONSUMER_GATE_CONTRACT);
    assert_eq!(decision.state, LakeCompletenessState::CompleteWithGaps);
    assert_eq!(decision.required_source_count, 1);
    assert_eq!(decision.complete_source_count, 0);
    assert_eq!(decision.missing_source_count, 1);
    assert_eq!(decision.quarantined_source_count, 0);
    assert_eq!(decision.manifest_digest, "a".repeat(64));
}

#[test]
fn consumers_refuse_unmatched_verification() {
    let err = ensure_lake_epoch_consumable(
        &epoch_row(LakeCompletenessState::Complete),
        &verification_row(LakeEpochVerificationStatus::Mismatch),
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )
    .expect_err("verification mismatch");

    assert!(matches!(
        err,
        LakeError::EpochVerificationNotMatched { epoch_id, .. } if epoch_id == "epoch-1"
    ));
}

#[test]
fn consumers_refuse_epoch_verification_boundary_mismatch() {
    let mut verification = verification_row(LakeEpochVerificationStatus::Match);
    verification.epoch_id = "epoch-2".to_string();

    let err = ensure_lake_epoch_consumable(
        &epoch_row(LakeCompletenessState::Complete),
        &verification,
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("boundary mismatch");

    assert!(matches!(
        err,
        LakeError::EpochVerificationBoundaryMismatch { epoch_id, verification_epoch_id }
            if epoch_id == "epoch-1" && verification_epoch_id == "epoch-2"
    ));
}

#[test]
fn consumers_refuse_padded_epoch_metadata_identity_before_release() {
    for field in [
        "epoch.epoch_id",
        "epoch.dataset_id",
        "epoch.policy",
        "epoch.opened_at",
        "epoch.sealed_at",
        "epoch.manifest_digest",
        "epoch.iceberg_snapshot_id",
        "verification.epoch_id",
        "verification.verification_id",
        "verification.completed_at",
    ] {
        let mut epoch = epoch_row(LakeCompletenessState::Complete);
        let mut verification = verification_row(LakeEpochVerificationStatus::Match);
        match field {
            "epoch.epoch_id" => epoch.epoch_id = " epoch-1".to_string(),
            "epoch.dataset_id" => epoch.dataset_id = "retail ".to_string(),
            "epoch.policy" => epoch.policy = " wait_all_required".to_string(),
            "epoch.opened_at" => epoch.opened_at = "open ".to_string(),
            "epoch.sealed_at" => epoch.sealed_at = " sealed".to_string(),
            "epoch.manifest_digest" => epoch.manifest_digest = " digest".to_string(),
            "epoch.iceberg_snapshot_id" => {
                epoch.iceberg_snapshot_id = Some(" snapshot-1".to_string());
            }
            "verification.epoch_id" => verification.epoch_id = "epoch-1 ".to_string(),
            "verification.verification_id" => {
                verification.verification_id =
                    " verify:retail:epoch-1:000000000000002a".to_string();
            }
            "verification.completed_at" => verification.completed_at = "done ".to_string(),
            _ => unreachable!("test fields are exhaustive"),
        }

        let err =
            ensure_lake_epoch_consumable(&epoch, &verification, LakeEpochConsumerOptions::strict())
                .expect_err("padded metadata rejected");

        assert!(matches!(
            err,
            LakeError::InvalidRawCdcEpochMetadataField { field: actual, reason }
                if actual == field && reason == "must not contain surrounding whitespace"
        ));
    }
}

#[test]
fn consumers_refuse_empty_epoch_metadata_identity_before_release() {
    let mut verification = verification_row(LakeEpochVerificationStatus::Match);
    verification.verification_id = " ".to_string();

    let err = ensure_lake_epoch_consumable(
        &epoch_row(LakeCompletenessState::Complete),
        &verification,
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("empty verification identity rejected");

    assert!(matches!(
        err,
        LakeError::InvalidRawCdcEpochMetadataField {
            field: "verification.verification_id",
            reason,
        } if reason == "must not be empty"
    ));
}

#[test]
fn consumers_refuse_verification_id_that_does_not_match_epoch_rollup() {
    let mut epoch = epoch_row(LakeCompletenessState::Complete);
    epoch.checksum_rollup = 43;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("verification id mismatch rejected");

    assert!(matches!(
        err,
        LakeError::InvalidRawCdcEpochMetadataField {
            field: "verification.verification_id",
            reason,
        } if reason == "must equal verify:retail:epoch-1:000000000000002b"
    ));
}

#[test]
fn consumers_refuse_manifest_digest_that_is_not_sha256_shaped() {
    for manifest_digest in ["digest", &"g".repeat(64), &"a".repeat(63)] {
        let mut epoch = epoch_row(LakeCompletenessState::Complete);
        epoch.manifest_digest = manifest_digest.to_string();

        let err = ensure_lake_epoch_consumable(
            &epoch,
            &verification_row(LakeEpochVerificationStatus::Match),
            LakeEpochConsumerOptions::strict(),
        )
        .expect_err("manifest digest shape rejected");

        assert!(matches!(
            err,
            LakeError::InvalidRawCdcEpochMetadataField {
                field: "epoch.manifest_digest",
                reason,
            } if reason == "must be a 64-character hex digest"
        ));
    }
}

#[test]
fn consumers_refuse_non_consumable_epoch_states() {
    let mut epoch = epoch_row(LakeCompletenessState::Quarantined);
    epoch.complete_source_count = 0;
    epoch.quarantined_source_count = 1;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )
    .expect_err("quarantined epoch");

    assert!(matches!(
        err,
        LakeError::EpochNotConsumable {
            state: LakeCompletenessState::Quarantined,
            ..
        }
    ));
}

#[test]
fn consumers_refuse_complete_epoch_with_missing_source_count() {
    let mut epoch = epoch_row(LakeCompletenessState::Complete);
    epoch.complete_source_count = 0;
    epoch.missing_source_count = 1;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("complete epoch with gap rejected");

    assert!(matches!(
        err,
        LakeError::EpochCompletenessStateMismatch {
            epoch_id,
            state: LakeCompletenessState::Complete,
            missing_source_count: 1,
            quarantined_source_count: 0,
        } if epoch_id == "epoch-1"
    ));
}

#[test]
fn consumers_refuse_complete_epoch_without_any_source_evidence() {
    let mut epoch = epoch_row(LakeCompletenessState::Complete);
    epoch.required_source_count = 0;
    epoch.complete_source_count = 0;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("complete epoch without source evidence rejected");

    assert!(matches!(
        err,
        LakeError::EpochCompletenessStateMismatch {
            epoch_id,
            state: LakeCompletenessState::Complete,
            missing_source_count: 0,
            quarantined_source_count: 0,
        } if epoch_id == "epoch-1"
    ));
}

#[test]
fn consumers_refuse_complete_with_gaps_without_missing_sources() {
    let mut epoch = epoch_row(LakeCompletenessState::CompleteWithGaps);
    epoch.policy = "publish_with_gaps".to_string();

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )
    .expect_err("gap epoch without gap rejected");

    assert!(matches!(
        err,
        LakeError::EpochCompletenessStateMismatch {
            state: LakeCompletenessState::CompleteWithGaps,
            missing_source_count: 0,
            quarantined_source_count: 0,
            ..
        }
    ));
}

#[test]
fn consumers_refuse_complete_with_gaps_when_source_is_quarantined() {
    let mut epoch = epoch_row(LakeCompletenessState::CompleteWithGaps);
    epoch.policy = "publish_with_gaps".to_string();
    epoch.complete_source_count = 0;
    epoch.missing_source_count = 0;
    epoch.quarantined_source_count = 1;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )
    .expect_err("quarantined gap epoch rejected");

    assert!(matches!(
        err,
        LakeError::EpochCompletenessStateMismatch {
            state: LakeCompletenessState::CompleteWithGaps,
            missing_source_count: 0,
            quarantined_source_count: 1,
            ..
        }
    ));
}

#[test]
fn consumers_refuse_epoch_with_inconsistent_source_count_rollup() {
    let mut epoch = epoch_row(LakeCompletenessState::Complete);
    epoch.required_source_count = 2;

    let err = ensure_lake_epoch_consumable(
        &epoch,
        &verification_row(LakeEpochVerificationStatus::Match),
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("source count rollup rejected");

    assert!(matches!(
        err,
        LakeError::EpochSourceCountMismatch {
            required_source_count: 2,
            complete_source_count: 1,
            missing_source_count: 0,
            quarantined_source_count: 0,
            ..
        }
    ));
}

#[test]
fn consumers_refuse_verification_count_mismatch() {
    let mut verification = verification_row(LakeEpochVerificationStatus::Match);
    verification.lake_change_count = 2;

    let err = ensure_lake_epoch_consumable(
        &epoch_row(LakeCompletenessState::Complete),
        &verification,
        LakeEpochConsumerOptions::strict(),
    )
    .expect_err("verification count mismatch rejected");

    assert!(matches!(
        err,
        LakeError::EpochVerificationCountMismatch {
            field: "change_count",
            epoch_count: 1,
            input_count: 1,
            lake_count: 2,
            ..
        }
    ));
}

#[test]
fn recovery_guidance_distinguishes_reseed_from_replay() {
    let reseed = lake_epoch_recovery_guidance(LakeCompletenessState::Reseeding);
    let replay = lake_epoch_recovery_guidance(LakeCompletenessState::FailedRecoverable);

    assert_eq!(reseed.code, "source_reseed_required");
    assert!(reseed.operator_action.contains("finish source reseed"));
    assert_eq!(replay.code, "writer_replay_required");
    assert!(replay.operator_action.contains("durable stream offsets"));
}

#[test]
fn recovery_guidance_requires_explicit_gap_acceptance() {
    let guidance = lake_epoch_recovery_guidance(LakeCompletenessState::CompleteWithGaps);

    assert_eq!(guidance.code, "explicit_gap_acceptance_required");
    assert!(guidance.consumer_gate.contains("explicitly accepts"));
    assert!(guidance
        .evidence_required
        .contains("--accept-complete-with-gaps"));
}

fn epoch_row(state: LakeCompletenessState) -> LakeRawCdcEpochRow {
    LakeRawCdcEpochRow {
        epoch_id: "epoch-1".to_string(),
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

fn verification_row(status: LakeEpochVerificationStatus) -> LakeRawCdcEpochVerificationRow {
    LakeRawCdcEpochVerificationRow {
        epoch_id: "epoch-1".to_string(),
        verification_id: "verify:retail:epoch-1:000000000000002a".to_string(),
        input_transaction_count: 1,
        input_change_count: 1,
        lake_transaction_count: 1,
        lake_change_count: 1,
        checksum_status: status,
        completed_at: "done".to_string(),
    }
}
