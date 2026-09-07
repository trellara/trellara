use crate::fleet_fanin::FleetFanInSimulationAction;
use crate::fleet_fanin_fixture::fleet_conflicting_duplicate;
use crate::fleet_fanin_steps::FleetFanInSteps;
use crate::fleet_fanin_workload::FleetFanInWorkload;
use trellara_protocol::TransactionEnvelope;

pub(crate) fn record_initial_epoch_inputs(
    workload: &FleetFanInWorkload,
    steps: &mut FleetFanInSteps,
) -> Vec<TransactionEnvelope> {
    let envelopes = workload.online_envelopes.clone();
    for envelope in &envelopes {
        steps.push(
            Some(envelope.source_id.clone()),
            FleetFanInSimulationAction::SourceEnvelopePublished,
        );
    }
    for source_id in workload.missing_sources().collect::<Vec<_>>() {
        steps.push(
            Some(source_id),
            FleetFanInSimulationAction::SourceMissingAtEpochSeal,
        );
    }
    envelopes
}

pub(crate) fn append_late_source_envelopes(
    workload: &FleetFanInWorkload,
    envelopes: &mut Vec<TransactionEnvelope>,
    steps: &mut FleetFanInSteps,
) {
    for envelope in workload.late_envelopes.clone() {
        steps.push(
            Some(envelope.source_id.clone()),
            FleetFanInSimulationAction::LateSourceEnvelopeArrived,
        );
        envelopes.push(envelope);
    }
}

pub(crate) fn append_duplicate_replays(
    workload: &FleetFanInWorkload,
    duplicate_replay_count: usize,
    envelopes: &mut Vec<TransactionEnvelope>,
    steps: &mut FleetFanInSteps,
) {
    for duplicate in workload
        .online_envelopes
        .iter()
        .take(duplicate_replay_count)
        .cloned()
    {
        steps.push(
            Some(duplicate.source_id.clone()),
            FleetFanInSimulationAction::SourceEnvelopeReplayedDuplicate,
        );
        envelopes.push(duplicate);
    }
}

pub(crate) fn append_conflicting_duplicate_replay(
    workload: &FleetFanInWorkload,
    envelopes: &mut Vec<TransactionEnvelope>,
    steps: &mut FleetFanInSteps,
) -> bool {
    let Some(first) = workload.online_envelopes.first() else {
        return false;
    };

    let conflicting = fleet_conflicting_duplicate(first);
    steps.push(
        Some(conflicting.source_id.clone()),
        FleetFanInSimulationAction::SourceEnvelopeReplayedDuplicate,
    );
    envelopes.push(conflicting);
    true
}

#[cfg(test)]
#[path = "fleet_fanin_envelopes_tests.rs"]
mod fleet_fanin_envelopes_tests;
