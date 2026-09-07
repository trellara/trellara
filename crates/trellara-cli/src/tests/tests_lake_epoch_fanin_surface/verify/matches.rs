use super::*;

#[test]
fn lake_fanin_verify_matches_identical_epoch_artifacts() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "match",
        |_| {},
        |_| {},
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Match);
    assert_eq!(summary.matched_check_count, 28);
    assert_eq!(summary.mismatch_count, 0);
    assert_eq!(summary.warning_mismatch_count, 0);
    assert_eq!(summary.blocker_mismatch_count, 0);
    assert!(summary.source_counts_match);
    assert_eq!(summary.stream_required_source_count, 12);
    assert_eq!(summary.stream_complete_source_count, 12);
    assert_eq!(summary.stream_missing_source_count, 0);
    assert_eq!(summary.stream_quarantined_source_count, 0);
    assert_eq!(summary.lake_required_source_count, 12);
    assert_eq!(summary.lake_complete_source_count, 12);
    assert_eq!(summary.lake_missing_source_count, 0);
    assert_eq!(summary.lake_quarantined_source_count, 0);
    assert!(summary.checksum_rollup_match);
    assert_eq!(summary.stream_checksum_rollup, summary.lake_checksum_rollup);
    assert!(summary.spark_consumption_allowed);
    assert_eq!(
        summary.spark_consumption_contract,
        trellara_lake::LAKE_EPOCH_CONSUMER_GATE_CONTRACT
    );
    assert!(summary
        .spark_consumption_gate
        .contains("released: stream and lake proofs match"));
    assert!(summary
        .recommended_next_steps
        .iter()
        .any(|step| step.contains("Spark templates")));
}

#[test]
fn lake_fanin_verify_refuses_complete_with_gaps_without_explicit_acceptance() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "gap-default",
        |_| {},
        |_| {},
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Match);
    assert_eq!(summary.mismatch_count, 0);
    assert!(!summary.spark_consumption_allowed);
    assert_eq!(
        summary.spark_consumption_gate,
        "blocked: complete_with_gaps requires --accept-complete-with-gaps before Spark consumption"
    );
    assert!(summary
        .recommended_next_steps
        .iter()
        .any(|step| step.contains("keep the epoch out of Spark")));
}

#[test]
fn lake_fanin_verify_releases_complete_with_gaps_with_explicit_acceptance() {
    let summary = verify_summary_with_gap_acceptance(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "gap-accepted",
        true,
        |_| {},
        |_| {},
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Match);
    assert!(summary.spark_consumption_allowed);
    assert!(summary
        .spark_consumption_gate
        .contains("complete_with_gaps was explicitly accepted"));
    assert!(summary
        .proof_command
        .contains("--accept-complete-with-gaps"));
}

#[test]
fn lake_fanin_verify_accepts_reordered_source_watermarks() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "source-order",
        |_| {},
        |lake| lake.source_watermarks.reverse(),
    );

    assert_matches_without_mismatches(&summary);
}

#[test]
fn lake_fanin_verify_accepts_reordered_table_rollups() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "table-order",
        |stream| {
            let mut extra_rollup = stream.table_rollups[0].clone();
            extra_rollup.relation = "public.refunds".to_string();
            stream.table_rollups.push(extra_rollup);
            refresh_manifest_digest(stream);
        },
        |lake| lake.table_rollups.reverse(),
    );

    assert_matches_without_mismatches(&summary);
}

#[test]
fn lake_fanin_verify_accepts_reordered_partition_rollups() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "partition-order",
        |_| {},
        |lake| lake.partition_rollups.reverse(),
    );

    assert_matches_without_mismatches(&summary);
}

#[test]
fn lake_fanin_verify_accepts_reordered_quarantine_entries() {
    let summary = verify_summary(
        LakeEpochScenario::ConflictingDuplicateQuarantine,
        "quarantine-order",
        |stream| {
            let mut extra_entry = stream.quarantine_entries[0].clone();
            extra_entry.transaction_id = Some("tx-conflicting-2".to_string());
            stream.quarantine_entries.push(extra_entry);
            refresh_manifest_digest(stream);
        },
        |lake| lake.quarantine_entries.reverse(),
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Match);
    assert_eq!(summary.mismatch_count, 0);
    assert_eq!(summary.warning_mismatch_count, 0);
    assert_eq!(summary.blocker_mismatch_count, 0);
    assert!(!summary.spark_consumption_allowed);
    assert!(summary
        .spark_consumption_gate
        .contains("lake epoch state quarantined is not consumable"));
}

#[test]
fn lake_fanin_verify_surfaces_reseed_recovery_action() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "reseed-guidance",
        |stream| {
            stream.state = trellara_lake::LakeCompletenessState::Reseeding;
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.state = trellara_lake::LakeCompletenessState::Reseeding;
            refresh_manifest_digest(lake);
        },
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Match);
    assert!(!summary.spark_consumption_allowed);
    assert!(summary
        .spark_consumption_gate
        .contains("source_reseed_required"));
    assert!(summary
        .spark_consumption_gate
        .contains("finish source reseed"));
}

#[test]
fn lake_fanin_verify_surfaces_replay_recovery_action() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "replay-guidance",
        |stream| {
            stream.state = trellara_lake::LakeCompletenessState::FailedRecoverable;
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.state = trellara_lake::LakeCompletenessState::FailedRecoverable;
            refresh_manifest_digest(lake);
        },
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Match);
    assert!(!summary.spark_consumption_allowed);
    assert!(summary
        .spark_consumption_gate
        .contains("writer_replay_required"));
    assert!(summary
        .spark_consumption_gate
        .contains("durable stream offsets"));
}
