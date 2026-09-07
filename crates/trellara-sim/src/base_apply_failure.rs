use crate::base::{FailurePoint, SimulationAction};
use crate::base_steps::BaseSimulationSteps;
use crate::base_stream::BaseStream;
use crate::base_target::BaseTarget;
use crate::failure_injection::FailureInjection;
use crate::transaction::Transaction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApplyFailure {
    None,
    RedeliveredBeforeCommit,
}

pub(crate) fn inject_pre_commit_apply_failure(
    failure: &mut FailureInjection,
    configured_failure: FailurePoint,
    transaction: Transaction,
    target: &mut BaseTarget,
    stream: &mut BaseStream,
    steps: &mut BaseSimulationSteps,
) -> (ApplyFailure, Option<Transaction>) {
    if failure.should_inject(configured_failure, FailurePoint::TargetFailureBeforeCommit) {
        failure.mark("target failed before committing the transaction");
        push_step(
            &transaction,
            steps,
            SimulationAction::TargetFailureBeforeCommit,
        );
        stream.redeliver_front(transaction);
        return (ApplyFailure::RedeliveredBeforeCommit, None);
    }

    if failure.should_inject(
        configured_failure,
        FailurePoint::TargetQuarantineRepairReplay,
    ) {
        failure.mark(
            "target apply failed closed into quarantine; operator repaired the contract and marked transaction replay-ready",
        );
        target.quarantine(transaction.id.clone());
        push_step(&transaction, steps, SimulationAction::TargetQuarantined);
        target.repair_quarantine(&transaction.id);
        push_step(
            &transaction,
            steps,
            SimulationAction::OperatorMarkedReplayReady,
        );
        stream.redeliver_front(transaction);
        return (ApplyFailure::RedeliveredBeforeCommit, None);
    }

    if failure.should_inject(
        configured_failure,
        FailurePoint::CheckpointFailureDuringApply,
    ) {
        failure.mark("target checkpoint write failed inside the apply transaction");
        push_step(
            &transaction,
            steps,
            SimulationAction::CheckpointFailureRolledBackApply,
        );
        stream.redeliver_front(transaction);
        return (ApplyFailure::RedeliveredBeforeCommit, None);
    }

    (ApplyFailure::None, Some(transaction))
}

fn push_step(transaction: &Transaction, steps: &mut BaseSimulationSteps, action: SimulationAction) {
    steps.push(transaction.id.clone(), transaction.lsn, action);
}

#[cfg(test)]
#[path = "base_apply_failure_tests.rs"]
mod base_apply_failure_tests;
