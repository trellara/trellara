use super::*;
use std::collections::HashSet;

#[test]
fn snapshot_suite_covers_every_required_failure_point() {
    let reports = run_default_snapshot_suite(42);
    let covered = reports
        .iter()
        .map(|report| report.failure_point)
        .collect::<HashSet<_>>();

    assert_eq!(reports.len(), SnapshotFailurePoint::ALL.len());
    assert!(SnapshotFailurePoint::ALL
        .into_iter()
        .all(|failure_point| covered.contains(&failure_point)));
    assert!(reports.iter().all(|report| report.passed));
}

#[test]
fn snapshot_source_crash_during_table_copy_retries_before_handoff() {
    let report = run_snapshot_simulation(SnapshotSimulationConfig::new(
        77,
        SnapshotFailurePoint::SourceCrashDuringTableCopy,
    ));

    assert!(report.passed);
    assert_eq!(report.table_count, report.copied_tables);
    assert!(report.handoff_recorded);
    assert!(report.verification_matched);
    let source_crash_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::SourceCrashedDuringTableCopy)
        .expect("source crash step");
    let retry_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::TableCopyRetried)
        .expect("retry step");
    let handoff_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::HandoffRecorded)
        .expect("handoff step");
    assert!(source_crash_index < retry_index);
    assert!(retry_index < handoff_index);
}

#[test]
fn snapshot_relay_crash_during_table_copy_retries_before_handoff() {
    let report = run_snapshot_simulation(SnapshotSimulationConfig::new(
        99,
        SnapshotFailurePoint::RelayCrashDuringTableCopy,
    ));

    assert!(report.passed);
    assert_eq!(report.table_count, report.copied_tables);
    assert!(report.handoff_recorded);
    assert!(report.stream_started);
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == SnapshotSimulationAction::TableCopyRetried));
    let failed_recoverable_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::SnapshotMarkedFailedRecoverable)
        .expect("failed recoverable marker");
    let retry_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::TableCopyRetried)
        .expect("retry step");
    assert!(failed_recoverable_index < retry_index);
    let handoff_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::HandoffRecorded)
        .expect("handoff step");
    let last_copy_index = report
        .steps
        .iter()
        .rposition(|step| step.action == SnapshotSimulationAction::TableCopyCompleted)
        .expect("copy complete step");
    assert!(last_copy_index < handoff_index);
}

#[test]
fn snapshot_target_crash_during_table_copy_retries_before_handoff() {
    let report = run_snapshot_simulation(SnapshotSimulationConfig::new(
        123,
        SnapshotFailurePoint::TargetCrashDuringTableCopy,
    ));

    assert!(report.passed);
    assert_eq!(report.table_count, report.copied_tables);
    assert!(report.verification_matched);
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == SnapshotSimulationAction::TableCopyCrashed));
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == SnapshotSimulationAction::SnapshotMarkedFailedRecoverable));
}

#[test]
fn snapshot_duplicate_copy_attempt_skips_completed_table_before_handoff() {
    let report = run_snapshot_simulation(SnapshotSimulationConfig::new(
        231,
        SnapshotFailurePoint::DuplicateCopyAttempt,
    ));

    assert!(report.passed);
    assert_eq!(report.table_count, report.copied_tables);
    assert!(report.handoff_recorded);
    let completed_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::TableCopyCompleted)
        .expect("table completed");
    let duplicate_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::DuplicateTableCopyAttempted)
        .expect("duplicate attempt");
    let skipped_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::DuplicateTableCopySkipped)
        .expect("duplicate skipped");
    let handoff_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::HandoffRecorded)
        .expect("handoff step");
    assert!(completed_index < duplicate_index);
    assert!(duplicate_index < skipped_index);
    assert!(skipped_index < handoff_index);
}

#[test]
fn snapshot_ddl_during_table_copy_withholds_handoff_until_contract_refresh() {
    let report = run_snapshot_simulation(SnapshotSimulationConfig::new(
        321,
        SnapshotFailurePoint::DdlDuringTableCopy,
    ));

    assert!(report.passed);
    assert!(report.contract_refreshed);
    assert!(report.handoff_recorded);
    assert!(report.verification_matched);
    let drift_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::SchemaDriftDetected)
        .expect("schema drift step");
    let withheld_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::HandoffWithheld)
        .expect("handoff withheld step");
    let refreshed_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::ContractRefreshed)
        .expect("contract refresh step");
    let handoff_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::HandoffRecorded)
        .expect("handoff step");
    assert!(drift_index < withheld_index);
    assert!(withheld_index < refreshed_index);
    assert!(refreshed_index < handoff_index);
}

#[test]
fn handoff_recorded_before_stream_start_recovers_before_replay() {
    let report = run_snapshot_simulation(SnapshotSimulationConfig::new(
        456,
        SnapshotFailurePoint::HandoffRecordedBeforeStreamStart,
    ));

    assert!(report.passed);
    assert!(report.handoff_recorded);
    assert!(report.stream_started);
    assert_eq!(
        report.writes_after_snapshot,
        report.stream_replayed_transactions
    );
    let crash_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::RelayCrashedBeforeStreamStart)
        .expect("crash step");
    let stream_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::StreamStarted)
        .expect("stream step");
    let first_replay_index = report
        .steps
        .iter()
        .position(|step| step.action == SnapshotSimulationAction::PostSnapshotWriteReplayed)
        .expect("replay step");
    assert!(crash_index < stream_index);
    assert!(stream_index < first_replay_index);
}
