use super::*;

#[test]
fn lake_fanin_verify_explains_blocked_spark_gate_for_proof_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "spark-gate-mismatch",
        |_| {},
        |lake| lake.transaction_count += 1,
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Blocked);
    assert!(!summary.spark_consumption_allowed);
    assert_eq!(
        summary.spark_consumption_gate,
        "blocked: stream and lake epoch proof artifacts do not match"
    );
}

#[test]
fn lake_fanin_verify_blocks_straggler_policy_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "policy",
        |_| {},
        |lake| lake.straggler_policy = "wait_all_required".to_string(),
    );

    assert_blocker(&summary, "straggler_policy");
}

#[test]
fn lake_fanin_verify_blocks_watermark_rollup_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "watermark-rollup",
        |_| {},
        |lake| lake.watermark_rollup.global_low_watermark_lsn = Some("0/DEADBEEF".to_string()),
    );

    assert_blocker(&summary, "watermark_rollup");
}

#[test]
fn lake_fanin_verify_blocks_source_watermark_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "source-watermark",
        |_| {},
        |lake| lake.source_watermarks[0].end_lsn = Some("0/DEADBEEF".to_string()),
    );

    assert_blocker(&summary, "source_watermarks");
}

#[test]
fn lake_fanin_verify_blocks_manifest_digest_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "digest",
        |_| {},
        |lake| lake.manifest_digest = "0".repeat(64),
    );

    assert_blocker(&summary, "manifest_digest");
}

#[test]
fn lake_fanin_verify_blocks_matching_artifacts_with_stale_manifest_digest() {
    let stale_stream_digest = "0".repeat(64);
    let stale_lake_digest = stale_stream_digest.clone();
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "stale-digest",
        |stream| stream.manifest_digest = stale_stream_digest,
        |lake| lake.manifest_digest = stale_lake_digest,
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("manifest_digest does not match epoch rows")
        || mismatch
            .lake_value
            .contains("manifest_digest does not match epoch rows")));
}

#[test]
fn lake_fanin_verify_blocks_count_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "blocked",
        |_| {},
        |lake| lake.transaction_count += 1,
    );

    assert_blocker(&summary, "transaction_count");
    assert!(summary
        .recommended_next_steps
        .iter()
        .any(|step| step.contains("_trellara_epochs")));
}

#[test]
fn lake_fanin_verify_surfaces_source_count_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "source-count-mismatch",
        |_| {},
        |lake| lake.required_source_count += 1,
    );

    assert_blocker(&summary, "required_source_count");
    assert!(!summary.source_counts_match);
    assert_eq!(summary.stream_required_source_count, 12);
    assert_eq!(summary.lake_required_source_count, 13);
}

#[test]
fn lake_fanin_verify_blocks_checksum_rollup_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "checksum-rollup",
        |_| {},
        |lake| lake.checksum_rollup ^= 1,
    );

    assert_blocker(&summary, "checksum_rollup");
    assert!(!summary.checksum_rollup_match);
    assert_ne!(summary.stream_checksum_rollup, summary.lake_checksum_rollup);
}
