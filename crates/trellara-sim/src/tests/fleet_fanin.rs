use super::*;
use trellara_lake::{LakeCompletenessState, LakeEpochVerificationStatus};

#[test]
fn fleet_fanin_offline_stores_publish_with_explicit_gap_state() {
    let report = run_fleet_fanin_simulation(FleetFanInSimulationConfig::new(
        42,
        FleetFanInFailurePoint::OfflineStoresPublishWithGaps,
    ));

    assert!(report.passed);
    assert_eq!(
        report.initial_state,
        LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(report.required_source_count, 12);
    assert_eq!(report.complete_source_count, 9);
    assert_eq!(report.missing_source_count, 3);
    assert_eq!(report.transaction_count, 9);
    assert_eq!(report.change_count, 9);
    assert_eq!(
        report.verification_status,
        LakeEpochVerificationStatus::Match
    );
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == FleetFanInSimulationAction::EpochPublishedWithGaps));
}

#[test]
fn fleet_fanin_late_sources_recompute_epoch_to_complete() {
    let report = run_fleet_fanin_simulation(FleetFanInSimulationConfig::new(
        42,
        FleetFanInFailurePoint::LateStoreRecoveryCompletesEpoch,
    ));

    assert!(report.passed);
    assert_eq!(
        report.initial_state,
        LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(
        report.recovered_state,
        Some(LakeCompletenessState::Complete)
    );
    assert_eq!(report.complete_source_count, 12);
    assert_eq!(report.missing_source_count, 0);
    assert_eq!(report.transaction_count, 12);
    assert_eq!(report.change_count, 12);
    assert_eq!(
        report
            .steps
            .iter()
            .filter(|step| { step.action == FleetFanInSimulationAction::LateSourceEnvelopeArrived })
            .count(),
        3
    );
}

#[test]
fn fleet_fanin_duplicate_store_replay_is_deduplicated() {
    let report = run_fleet_fanin_simulation(FleetFanInSimulationConfig::new(
        42,
        FleetFanInFailurePoint::DuplicateStoreTransactionReplay,
    ));

    assert!(report.passed);
    assert_eq!(report.duplicate_replay_count, 2);
    assert_eq!(report.transaction_count, 9);
    assert_eq!(report.change_count, 9);
    assert_eq!(
        report.initial_state,
        LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(
        report
            .steps
            .iter()
            .filter(|step| {
                step.action == FleetFanInSimulationAction::SourceEnvelopeReplayedDuplicate
            })
            .count(),
        2
    );
}

#[test]
fn fleet_fanin_conflicting_duplicate_quarantines_epoch() {
    let report = run_fleet_fanin_simulation(FleetFanInSimulationConfig::new(
        42,
        FleetFanInFailurePoint::ConflictingDuplicateQuarantine,
    ));

    assert!(report.passed);
    assert_eq!(report.initial_state, LakeCompletenessState::Quarantined);
    assert_eq!(
        report.complete_source_count
            + report.quarantined_source_count
            + report.missing_source_count,
        report.required_source_count
    );
    assert_eq!(report.quarantined_source_count, 1);
    assert!(report
        .source_watermarks
        .iter()
        .any(|source| source.source_id == "store-0001" && source.state == "quarantined"));
    assert_eq!(
        report.verification_status,
        LakeEpochVerificationStatus::Unknown
    );
    assert!(report
        .injected_failure
        .as_deref()
        .is_some_and(|failure| failure.contains("conflicting")));
    assert_eq!(report.quarantine_entries.len(), 1);
    assert_eq!(report.quarantine_entries[0].source_id, "store-0001");
    assert_eq!(
        report.quarantine_entries[0].reason,
        "conflicting_duplicate_idempotency"
    );
    assert!(report.quarantine_entries[0]
        .recovery_command
        .contains("trellara lake fanin verify"));
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == FleetFanInSimulationAction::ConflictingDuplicateDetected));
    assert!(report
        .steps
        .iter()
        .any(|step| step.action == FleetFanInSimulationAction::EpochQuarantined));
}
