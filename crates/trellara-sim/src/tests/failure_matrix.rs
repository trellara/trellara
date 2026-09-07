use super::*;
use std::collections::HashSet;

#[test]
fn default_suite_covers_every_required_failure_point() {
    let reports = run_default_suite(42);
    let covered = reports
        .iter()
        .map(|report| report.failure_point)
        .collect::<HashSet<_>>();

    assert_eq!(reports.len(), FailurePoint::ALL.len());
    assert!(FailurePoint::ALL
        .into_iter()
        .all(|failure_point| covered.contains(&failure_point)));
    assert!(reports.iter().all(|report| report.passed));
}

#[test]
fn simulations_are_replayable_by_seed_and_failure_point() {
    let config = SimulationConfig::new(17, FailurePoint::StreamAckLossAfterApply);

    assert_eq!(run_simulation(config), run_simulation(config));
}

#[test]
fn publish_ack_loss_recovers_by_republishing_without_missing_target_apply() {
    let report = run_simulation(SimulationConfig::new(99, FailurePoint::PublishAckLoss));

    assert!(report.passed);
    assert_eq!(report.transaction_count, report.applied_transactions);
    assert_eq!(report.skipped_duplicates, 1);
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == SimulationAction::PublishAckLost));
}

#[test]
fn checkpoint_failure_rolls_back_apply_until_recovery_redelivery() {
    let report = run_simulation(SimulationConfig::new(
        123,
        FailurePoint::CheckpointFailureDuringApply,
    ));

    assert!(report.passed);
    assert_eq!(report.transaction_count, report.applied_transactions);
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == SimulationAction::CheckpointFailureRolledBackApply));
}

#[test]
fn target_quarantine_repair_replay_applies_after_operator_marks_replay_ready() {
    let report = run_simulation(SimulationConfig::new(
        234,
        FailurePoint::TargetQuarantineRepairReplay,
    ));

    assert!(report.passed);
    assert_eq!(report.transaction_count, report.applied_transactions);
    assert_eq!(report.target_applied_lsn, report.relay_durable_lsn);
    assert_eq!(report.target_applied_lsn, report.source_acknowledged_lsn);
    let quarantine_index = report
        .steps
        .iter()
        .position(|step| step.action == SimulationAction::TargetQuarantined)
        .expect("quarantine step");
    let replay_ready_index = report
        .steps
        .iter()
        .position(|step| step.action == SimulationAction::OperatorMarkedReplayReady)
        .expect("replay-ready step");
    let apply_index = report
        .steps
        .iter()
        .position(|step| step.action == SimulationAction::AppliedAndCheckpointed)
        .expect("apply step");
    assert!(quarantine_index < replay_ready_index);
    assert!(replay_ready_index < apply_index);
}

#[test]
fn stream_ack_loss_after_apply_is_absorbed_by_dedup() {
    let report = run_simulation(SimulationConfig::new(
        456,
        FailurePoint::StreamAckLossAfterApply,
    ));

    assert!(report.passed);
    assert_eq!(report.transaction_count, report.applied_transactions);
    assert_eq!(report.skipped_duplicates, 1);
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == SimulationAction::StreamAckLost));
}

#[test]
fn source_failover_after_publish_before_ack_recovers_with_duplicate_replay() {
    let report = run_simulation(SimulationConfig::new(
        789,
        FailurePoint::SourceFailoverAfterPublishBeforeAck,
    ));

    assert!(report.passed);
    assert_eq!(report.transaction_count, report.applied_transactions);
    assert_eq!(report.skipped_duplicates, 1);
    assert_eq!(report.target_applied_lsn, report.relay_durable_lsn);
    assert_eq!(report.target_applied_lsn, report.source_acknowledged_lsn);
    assert!(report
        .steps
        .iter()
        .any(|step| { step.action == SimulationAction::SourceFailoverBeforeFeedback }));
}
