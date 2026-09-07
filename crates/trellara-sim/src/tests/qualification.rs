use super::*;
use std::collections::HashSet;

#[test]
fn default_qualification_suite_covers_lane_d_failure_points() {
    let reports = run_default_qualification_suite(42);
    let covered = reports
        .iter()
        .map(|report| report.failure_point)
        .collect::<HashSet<_>>();

    assert_eq!(reports.len(), QualificationFailurePoint::ALL.len());
    assert!(QualificationFailurePoint::ALL
        .into_iter()
        .all(|failure_point| covered.contains(&failure_point)));
    assert!(reports.iter().all(|report| report.passed));
    assert!(reports.iter().all(|report| report
        .observability_assertions
        .iter()
        .all(|assertion| assertion.passed)));
}

#[test]
fn qualification_source_promotion_while_relay_disconnected_recovers_via_failover_slot() {
    let report = run_qualification_simulation(QualificationSimulationConfig::new(
        99,
        QualificationFailurePoint::SourcePromotionWhileRelayDisconnected,
    ));

    assert!(report.passed);
    assert_eq!(
        report.durable_boundary,
        "source_failover_slot_to_relay_restart"
    );
    assert_eq!(
        report.invariant,
        "promoted_source_replays_from_last_durable_ack"
    );
    assert_eq!(report.duplicate_replays, 1);
    assert_eq!(report.source_acknowledged_lsn, report.durable_lsn);
    assert_eq!(report.target_applied_lsn, report.durable_lsn);
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == QualificationSimulationAction::RelayDisconnected));
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == QualificationSimulationAction::SourcePromoted));
    assert_observability(&report, "source_failover_promotion_detected");
    assert_observability(&report, "source_ack_lag_visible");
}

#[test]
fn qualification_broker_outage_quorum_loss_withholds_source_ack() {
    let report = run_qualification_simulation(QualificationSimulationConfig::new(
        100,
        QualificationFailurePoint::BrokerOutageQuorumLoss,
    ));

    assert!(report.passed);
    assert_eq!(report.durable_boundary, "broker_quorum_publish_ack");
    assert_eq!(report.invariant, "source_ack_waits_for_broker_quorum");
    let quorum_loss_index = step_index(&report, QualificationSimulationAction::BrokerQuorumLost);
    let source_ack_index = step_index(&report, QualificationSimulationAction::SourceAcked);
    assert!(quorum_loss_index < source_ack_index);
    assert_observability(&report, "broker_quorum_unavailable");
    assert_observability(&report, "publish_retry_recovered");
}

#[test]
fn qualification_target_restart_during_apply_replays_once() {
    let report = run_qualification_simulation(QualificationSimulationConfig::new(
        101,
        QualificationFailurePoint::TargetRestartDuringApply,
    ));

    assert!(report.passed);
    assert_eq!(report.durable_boundary, "target_apply_transaction_commit");
    assert_eq!(
        report.invariant,
        "target_restart_replays_uncheckpointed_apply"
    );
    assert_eq!(report.duplicate_replays, 1);
    let restart_index = step_index(
        &report,
        QualificationSimulationAction::TargetRestartedBeforeCommit,
    );
    let rollback_index = step_index(
        &report,
        QualificationSimulationAction::UncheckpointedApplyRolledBack,
    );
    let apply_index = step_index(
        &report,
        QualificationSimulationAction::AppliedAndCheckpointed,
    );
    assert!(restart_index < rollback_index);
    assert!(rollback_index < apply_index);
    assert_observability(&report, "target_restart_replay_required");
    assert_observability(&report, "target_checkpoint_not_advanced_early");
}

#[test]
fn qualification_object_store_success_catalog_timeout_retries_catalog_before_visibility() {
    let report = run_qualification_simulation(QualificationSimulationConfig::new(
        102,
        QualificationFailurePoint::ObjectStoreSuccessCatalogTimeout,
    ));

    assert!(report.passed);
    assert_eq!(
        report.durable_boundary,
        "object_store_write_before_catalog_visibility"
    );
    assert_eq!(
        report.invariant,
        "catalog_timeout_cannot_publish_uncommitted_epoch"
    );
    let object_store_index = step_index(
        &report,
        QualificationSimulationAction::ObjectStoreWriteSucceeded,
    );
    let catalog_timeout_index = step_index(
        &report,
        QualificationSimulationAction::CatalogCommitTimedOut,
    );
    let catalog_retry_index =
        step_index(&report, QualificationSimulationAction::CatalogCommitRetried);
    let source_ack_index = step_index(&report, QualificationSimulationAction::SourceAcked);
    assert!(object_store_index < catalog_timeout_index);
    assert!(catalog_timeout_index < catalog_retry_index);
    assert!(catalog_retry_index < source_ack_index);
    assert_observability(&report, "catalog_commit_pending");
    assert_observability(&report, "catalog_retry_idempotent");
}

#[test]
fn qualification_twenty_four_hour_soak_large_transaction_memory_ceiling() {
    let report = run_qualification_simulation(QualificationSimulationConfig::new(
        103,
        QualificationFailurePoint::TwentyFourHourSoakLargeTransactionMemoryCeiling,
    ));

    assert!(report.passed);
    assert_eq!(
        report.durable_boundary,
        "large_transaction_stream_spill_memory_ceiling"
    );
    assert_eq!(
        report.invariant,
        "soak_large_transactions_stay_within_memory_ceiling"
    );
    assert_eq!(report.soak_hours, 24);
    assert_eq!(report.large_transaction_change_count, 50_000);
    assert!(report.peak_memory_mib <= report.memory_ceiling_mib);
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == QualificationSimulationAction::LargeTransactionSpilled));
    assert_observability(&report, "soak_window_completed");
    assert_observability(&report, "large_transaction_memory_ceiling");
}

fn assert_observability(report: &QualificationSimulationReport, code: &str) {
    assert!(report
        .observability_assertions
        .iter()
        .any(|assertion| assertion.code == code && assertion.passed));
}

fn step_index(
    report: &QualificationSimulationReport,
    action: QualificationSimulationAction,
) -> usize {
    report
        .steps
        .iter()
        .position(|step| step.action == action)
        .unwrap_or_else(|| panic!("missing {action:?} step"))
}
