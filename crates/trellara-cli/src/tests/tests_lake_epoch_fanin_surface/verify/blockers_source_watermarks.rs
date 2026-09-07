use super::*;

#[test]
fn lake_fanin_verify_blocks_matching_artifacts_with_invalid_source_count_rollup() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "invalid-rollup",
        |stream| stream.required_source_count += 1,
        |lake| lake.required_source_count += 1,
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(!summary.source_counts_match);
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("source counts inconsistent")
        || mismatch.lake_value.contains("source counts inconsistent")));
}

#[test]
fn lake_fanin_verify_blocks_matching_artifacts_with_unknown_source_state() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "invalid-source-state",
        |stream| stream.source_watermarks[0].state = "mystery".to_string(),
        |lake| lake.source_watermarks[0].state = "mystery".to_string(),
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("unknown watermark state")
        || mismatch.lake_value.contains("unknown watermark state")));
}

#[test]
fn lake_fanin_verify_blocks_duplicate_source_watermark_identity() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "duplicate-source-watermark",
        |stream| {
            stream.source_watermarks[1].source_id = stream.source_watermarks[0].source_id.clone();
            stream.watermark_rollup = lake_epoch_watermark_rollup(&stream.source_watermarks);
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.source_watermarks[1].source_id = lake.source_watermarks[0].source_id.clone();
            lake.watermark_rollup = lake_epoch_watermark_rollup(&lake.source_watermarks);
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("duplicate source_id")
        || mismatch.lake_value.contains("duplicate source_id")));
}

#[test]
fn lake_fanin_verify_blocks_complete_source_without_end_lsn_evidence() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "missing-source-end-lsn",
        |stream| stream.source_watermarks[0].end_lsn = None,
        |lake| lake.source_watermarks[0].end_lsn = None,
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("requires end_lsn evidence")
        || mismatch.lake_value.contains("requires end_lsn evidence")));
}

#[test]
fn lake_fanin_verify_blocks_matching_complete_epoch_with_gap_counts() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "invalid-complete-gap",
        |stream| {
            stream.complete_source_count -= 1;
            stream.missing_source_count = 1;
            stream.source_watermarks[0].state = "missing".to_string();
            stream.source_watermarks[0].start_lsn = None;
            stream.source_watermarks[0].end_lsn = None;
            stream.source_watermarks[0].gap_reason =
                Some("required source missing from published gap epoch".to_string());
            stream.watermark_rollup = lake_epoch_watermark_rollup(&stream.source_watermarks);
        },
        |lake| {
            lake.complete_source_count -= 1;
            lake.missing_source_count = 1;
            lake.source_watermarks[0].state = "missing".to_string();
            lake.source_watermarks[0].start_lsn = None;
            lake.source_watermarks[0].end_lsn = None;
            lake.source_watermarks[0].gap_reason =
                Some("required source missing from published gap epoch".to_string());
            lake.watermark_rollup = lake_epoch_watermark_rollup(&lake.source_watermarks);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| {
        mismatch
            .stream_value
            .contains("state complete inconsistent")
            || mismatch.lake_value.contains("state complete inconsistent")
    }));
}

#[test]
fn lake_fanin_verify_blocks_matching_artifacts_with_source_watermark_count_drift() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "invalid-source-counts",
        |stream| {
            stream.source_watermarks[0].state = "missing".to_string();
            stream.source_watermarks[0].gap_reason =
                Some("required source missing from published gap epoch".to_string());
        },
        |lake| {
            lake.source_watermarks[0].state = "missing".to_string();
            lake.source_watermarks[0].gap_reason =
                Some("required source missing from published gap epoch".to_string());
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("source watermark rollup inconsistent")
        || mismatch
            .lake_value
            .contains("source watermark rollup inconsistent")));
}

#[test]
fn lake_fanin_verify_blocks_matching_artifacts_with_stale_watermark_rollup() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "invalid-stale-rollup",
        |stream| stream.watermark_rollup.global_low_watermark_lsn = Some("0/DEADBEEF".to_string()),
        |lake| lake.watermark_rollup.global_low_watermark_lsn = Some("0/DEADBEEF".to_string()),
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("stored watermark_rollup")
        || mismatch.lake_value.contains("stored watermark_rollup")));
}
