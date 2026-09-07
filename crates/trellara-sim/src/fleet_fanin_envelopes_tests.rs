use super::*;
use crate::fleet_fanin::{FleetFanInFailurePoint, FleetFanInSimulationConfig};

#[test]
fn initial_epoch_inputs_record_online_then_missing_sources() {
    let mut config =
        FleetFanInSimulationConfig::new(42, FleetFanInFailurePoint::OfflineStoresPublishWithGaps);
    config.store_count = 4;
    config.offline_store_count = 2;
    let workload = FleetFanInWorkload::new(&config);
    let mut steps = FleetFanInSteps::default();

    let envelopes = record_initial_epoch_inputs(&workload, &mut steps);

    assert_eq!(envelopes.len(), 2);
    assert_eq!(
        steps
            .into_steps()
            .into_iter()
            .map(|step| step.action)
            .collect::<Vec<_>>(),
        vec![
            FleetFanInSimulationAction::SourceEnvelopePublished,
            FleetFanInSimulationAction::SourceEnvelopePublished,
            FleetFanInSimulationAction::SourceMissingAtEpochSeal,
            FleetFanInSimulationAction::SourceMissingAtEpochSeal,
        ]
    );
}

#[test]
fn duplicate_replays_append_existing_online_envelopes() {
    let mut config = FleetFanInSimulationConfig::new(
        42,
        FleetFanInFailurePoint::DuplicateStoreTransactionReplay,
    );
    config.store_count = 4;
    config.offline_store_count = 1;
    config.duplicate_replay_count = 2;
    let workload = FleetFanInWorkload::new(&config);
    let mut steps = FleetFanInSteps::default();
    let mut envelopes = record_initial_epoch_inputs(&workload, &mut steps);

    append_duplicate_replays(
        &workload,
        config.duplicate_replay_count,
        &mut envelopes,
        &mut steps,
    );

    assert_eq!(envelopes.len(), 5);
    assert_eq!(
        steps
            .into_steps()
            .into_iter()
            .filter(|step| {
                step.action == FleetFanInSimulationAction::SourceEnvelopeReplayedDuplicate
            })
            .count(),
        2
    );
}

#[test]
fn conflicting_duplicate_replay_reuses_transaction_boundary_identity() {
    let mut config =
        FleetFanInSimulationConfig::new(42, FleetFanInFailurePoint::ConflictingDuplicateQuarantine);
    config.store_count = 3;
    config.offline_store_count = 1;
    let workload = FleetFanInWorkload::new(&config);
    let mut steps = FleetFanInSteps::default();
    let mut envelopes = record_initial_epoch_inputs(&workload, &mut steps);

    assert!(append_conflicting_duplicate_replay(
        &workload,
        &mut envelopes,
        &mut steps
    ));

    let original = workload.online_envelopes.first().expect("online envelope");
    let conflicting = envelopes.last().expect("conflicting replay");
    assert_eq!(conflicting.source_id, original.source_id);
    assert_eq!(conflicting.dataset_id, original.dataset_id);
    assert_eq!(conflicting.transaction_id, original.transaction_id);
    assert_eq!(conflicting.commit_lsn, original.commit_lsn);
    assert_eq!(
        conflicting.changes[0].idempotency_key,
        original.changes[0].idempotency_key
    );
    assert_ne!(conflicting.checksum, original.checksum);
    assert_eq!(
        steps
            .into_steps()
            .into_iter()
            .filter(|step| {
                step.action == FleetFanInSimulationAction::SourceEnvelopeReplayedDuplicate
            })
            .count(),
        1
    );
}
