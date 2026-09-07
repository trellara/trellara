use super::*;

#[test]
fn lake_fanin_verify_blocks_quarantine_entry_mismatch() {
    let summary = verify_summary(
        LakeEpochScenario::ConflictingDuplicateQuarantine,
        "quarantine",
        |_| {},
        |lake| lake.quarantine_entries.clear(),
    );

    assert_blocker(&summary, "quarantine_entries");
}

#[test]
fn lake_fanin_verify_blocks_duplicate_quarantine_entry_identity() {
    let summary = verify_summary(
        LakeEpochScenario::ConflictingDuplicateQuarantine,
        "duplicate-quarantine-entry",
        |stream| {
            let duplicate = stream.quarantine_entries[0].clone();
            stream.quarantine_entries.push(duplicate);
            refresh_manifest_digest(stream);
        },
        |lake| {
            let duplicate = lake.quarantine_entries[0].clone();
            lake.quarantine_entries.push(duplicate);
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("duplicate identity")
        || mismatch.lake_value.contains("duplicate identity")));
}

#[test]
fn lake_fanin_verify_blocks_invalid_quarantine_commit_lsn() {
    let summary = verify_summary(
        LakeEpochScenario::ConflictingDuplicateQuarantine,
        "invalid-quarantine-lsn",
        |stream| {
            stream.quarantine_entries[0].commit_lsn = Some("0/0".to_string());
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.quarantine_entries[0].commit_lsn = Some("0/0".to_string());
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("invalid commit_lsn")
        || mismatch.lake_value.contains("invalid commit_lsn")));
}

#[test]
fn lake_fanin_verify_blocks_quarantine_entry_without_quarantined_source() {
    let summary = verify_summary(
        LakeEpochScenario::ConflictingDuplicateQuarantine,
        "orphan-quarantine-entry",
        |stream| {
            stream.quarantine_entries[0].source_id = "store-9999".to_string();
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.quarantine_entries[0].source_id = "store-9999".to_string();
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("does not match a quarantined source watermark")
        || mismatch
            .lake_value
            .contains("does not match a quarantined source watermark")));
}

#[test]
fn lake_fanin_verify_blocks_quarantined_source_without_matching_entry() {
    let summary = verify_summary(
        LakeEpochScenario::ConflictingDuplicateQuarantine,
        "missing-quarantine-source-entry",
        |stream| {
            stream.source_watermarks[1].state = "quarantined".to_string();
            stream.source_watermarks[1].gap_reason =
                Some("operator marked source as unsafe".to_string());
            stream.quarantined_source_count += 1;
            stream.complete_source_count -= 1;
            stream.watermark_rollup = lake_epoch_watermark_rollup(&stream.source_watermarks);
            refresh_manifest_digest(stream);
        },
        |lake| {
            lake.source_watermarks[1].state = "quarantined".to_string();
            lake.source_watermarks[1].gap_reason =
                Some("operator marked source as unsafe".to_string());
            lake.quarantined_source_count += 1;
            lake.complete_source_count -= 1;
            lake.watermark_rollup = lake_epoch_watermark_rollup(&lake.source_watermarks);
            refresh_manifest_digest(lake);
        },
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("is missing quarantine entry evidence")
        || mismatch
            .lake_value
            .contains("is missing quarantine entry evidence")));
}

#[test]
fn lake_fanin_verify_blocks_quarantined_epoch_without_quarantine_entries() {
    let summary = verify_summary(
        LakeEpochScenario::ConflictingDuplicateQuarantine,
        "invalid-missing-quarantine-entry",
        |stream| stream.quarantine_entries.clear(),
        |lake| lake.quarantine_entries.clear(),
    );

    assert_blocker(&summary, "stream_epoch_consistency");
    assert_blocker(&summary, "lake_epoch_consistency");
    assert!(summary.mismatches.iter().any(|mismatch| mismatch
        .stream_value
        .contains("quarantined sources require quarantine entries")
        || mismatch
            .lake_value
            .contains("quarantined sources require quarantine entries")));
}
