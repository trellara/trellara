use super::*;

#[test]
fn lake_fanin_verify_blocks_table_rollup_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "table",
        |_| {},
        |lake| lake.table_rollups[0].change_count += 1,
    );

    assert_blocker(&summary, "table_rollups");
}

#[test]
fn lake_fanin_verify_blocks_duplicate_table_rollup_identity() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "duplicate-table-rollup",
        |stream| {
            let duplicate = stream.table_rollups[0].clone();
            stream.table_rollups.push(duplicate);
            refresh_manifest_digest(stream);
        },
        |lake| {
            let duplicate = lake.table_rollups[0].clone();
            lake.table_rollups.push(duplicate);
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("duplicate relation")
        || mismatch.lake_value.contains("duplicate relation")));
}

#[test]
fn lake_fanin_verify_blocks_table_rollup_with_changes_without_transactions() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "table-changes-without-transactions",
        |stream| {
            stream.table_rollups[0].transaction_count = 0;
            stream.table_rollups[0].change_count = 1;
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.table_rollups[0].transaction_count = 0;
            lake.table_rollups[0].change_count = 1;
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("changes without transactions")
        || mismatch.lake_value.contains("changes without transactions")));
}

#[test]
fn lake_fanin_verify_blocks_partition_rollup_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "partition",
        |_| {},
        |lake| lake.partition_rollups[0].event_count += 1,
    );

    assert_blocker(&summary, "partition_rollups");
}

#[test]
fn lake_fanin_verify_blocks_stale_partition_skew() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "stale-partition-skew",
        |_| {},
        |lake| {
            lake.partition_skew.max_event_count += 1;
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .lake_value
        .contains("stored partition_skew")
        || mismatch.stream_value.contains("stored partition_skew")));
}

#[test]
fn lake_fanin_verify_blocks_duplicate_partition_rollup_identity() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "duplicate-partition-rollup",
        |stream| {
            stream.partition_rollups[1].source_id = stream.partition_rollups[0].source_id.clone();
            stream.partition_rollups[1].partition_id = stream.partition_rollups[0].partition_id;
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.partition_rollups[1].source_id = lake.partition_rollups[0].source_id.clone();
            lake.partition_rollups[1].partition_id = lake.partition_rollups[0].partition_id;
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
fn lake_fanin_verify_blocks_invalid_partition_rollup_lsn_window() {
    let summary = verify_summary(
        LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
        "invalid-partition-lsn-window",
        |stream| {
            stream.partition_rollups[0].first_commit_lsn = Some("0/16B6FFF".to_string());
            stream.partition_rollups[0].last_commit_lsn = Some("0/16B6C50".to_string());
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.partition_rollups[0].first_commit_lsn = Some("0/16B6FFF".to_string());
            lake.partition_rollups[0].last_commit_lsn = Some("0/16B6C50".to_string());
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("first_commit_lsn after last_commit_lsn")
        || mismatch
            .lake_value
            .contains("first_commit_lsn after last_commit_lsn")));
}
