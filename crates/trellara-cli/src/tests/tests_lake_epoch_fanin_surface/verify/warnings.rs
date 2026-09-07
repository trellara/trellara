use super::*;

#[test]
fn lake_fanin_verify_reports_warning_only_mismatches_without_blocking_proof_repair() {
    let summary = verify_summary(
        LakeEpochScenario::OfflineStoresPublishWithGaps,
        "warning-only",
        |_| {},
        |lake| lake.duplicate_replay_count += 1,
    );

    assert_eq!(summary.status, LakeFaninVerifyStatus::Mismatch);
    assert_eq!(summary.mismatch_count, 1);
    assert_eq!(summary.warning_mismatch_count, 1);
    assert_eq!(summary.blocker_mismatch_count, 0);
    assert_eq!(summary.matched_check_count, 27);
    assert!(!summary.spark_consumption_allowed);
    assert_eq!(
        summary.spark_consumption_gate,
        "blocked: stream and lake epoch proof artifacts do not match"
    );
    assert!(summary.mismatches.iter().any(|mismatch| {
        mismatch.field == "duplicate_replay_count"
            && mismatch.severity == LakeFaninVerifyMismatchSeverity::Warning
    }));
    assert!(summary
        .recommended_next_steps
        .iter()
        .any(|step| step.contains("warning mismatches")));
}
