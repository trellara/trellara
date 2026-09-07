use super::*;

#[test]
fn run_summary_marks_snapshot_verified_after_converged_verification() {
    let mut snapshot = fixture_snapshot_copy_summary();
    let verify = matching_verify_summary();

    mark_snapshot_summary_verified(Some(&mut snapshot), Some(&verify));

    assert_eq!(snapshot.state, "verified");
    assert!(snapshot
        .consistency_note
        .contains("promoted the snapshot run to verified"));
}

#[test]
fn run_summary_keeps_snapshot_handoff_ready_without_converged_verification() {
    let mut snapshot = fixture_snapshot_copy_summary();
    let verify = mismatched_verify_summary();

    mark_snapshot_summary_verified(Some(&mut snapshot), Some(&verify));

    assert_eq!(snapshot.state, "stream_handoff_ready");
    assert!(!snapshot
        .consistency_note
        .contains("promoted the snapshot run to verified"));

    mark_snapshot_summary_verified(Some(&mut snapshot), None);
    assert_eq!(snapshot.state, "stream_handoff_ready");
}

#[test]
fn snapshot_copy_failure_records_recoverable_run_and_table_state() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse config");

    let (run, progress) = snapshot_copy_failure_records(
        &config,
        "snapshot-run-1",
        "trellara_slot",
        "0/16B8000",
        "public.sales",
        10,
        "target connection dropped during table copy".to_string(),
    );

    assert_eq!(run.source_id, "local-source");
    assert_eq!(run.dataset_id, "retail-sales");
    assert_eq!(run.run_id, "snapshot-run-1");
    assert_eq!(run.state, SnapshotRunState::FailedRecoverable);
    assert_eq!(run.current_relation.as_deref(), Some("public.sales"));
    assert_eq!(run.consistent_lsn.as_deref(), Some("0/16B8000"));
    assert_eq!(run.copied_rows, 10);
    assert_eq!(
        run.failure_reason.as_deref(),
        Some("target connection dropped during table copy")
    );
    assert_eq!(progress.source_id, "local-source");
    assert_eq!(progress.dataset_id, "retail-sales");
    assert_eq!(progress.run_id, "snapshot-run-1");
    assert_eq!(progress.relation, "public.sales");
    assert_eq!(progress.state, SnapshotRunState::FailedRecoverable);
    assert_eq!(progress.watermark_lsn.as_deref(), Some("0/16B8000"));
}
