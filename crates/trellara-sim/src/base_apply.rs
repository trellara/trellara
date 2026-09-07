use crate::base::{FailurePoint, SimulationAction};
use crate::base_apply_failure::{inject_pre_commit_apply_failure, ApplyFailure};
use crate::base_steps::BaseSimulationSteps;
use crate::base_stream::BaseStream;
use crate::base_target::BaseTarget;
use crate::failure_injection::FailureInjection;
use crate::transaction::Transaction;

pub(crate) fn apply_next_message(
    stream: &mut BaseStream,
    target: &mut BaseTarget,
    failure: &mut FailureInjection,
    configured_failure: FailurePoint,
    steps: &mut BaseSimulationSteps,
) -> Option<u64> {
    let transaction = stream.pop_next()?;
    push_step(&transaction, steps, SimulationAction::Delivered);

    if target.has_applied(&transaction.id) {
        target.mark_duplicate_skipped();
        stream.acknowledge();
        push_step(&transaction, steps, SimulationAction::SkippedDuplicate);
        push_step(&transaction, steps, SimulationAction::StreamAcked);
        return None;
    }

    let (apply_failure, transaction) = inject_pre_commit_apply_failure(
        failure,
        configured_failure,
        transaction,
        target,
        stream,
        steps,
    );
    if apply_failure == ApplyFailure::RedeliveredBeforeCommit {
        return None;
    }
    let transaction = transaction.expect("transaction preserved when no apply failure injected");

    target.apply(transaction.id.clone());
    push_step(
        &transaction,
        steps,
        SimulationAction::AppliedAndCheckpointed,
    );
    let applied_lsn = transaction.lsn;

    if failure.should_inject(configured_failure, FailurePoint::StreamAckLossAfterApply) {
        failure.mark("stream acknowledgement failed after target commit");
        push_step(&transaction, steps, SimulationAction::StreamAckLost);
        stream.redeliver_back(transaction);
        return Some(applied_lsn);
    }

    stream.acknowledge();
    push_step(&transaction, steps, SimulationAction::StreamAcked);
    Some(applied_lsn)
}

fn push_step(transaction: &Transaction, steps: &mut BaseSimulationSteps, action: SimulationAction) {
    steps.push(transaction.id.clone(), transaction.lsn, action);
}

#[cfg(test)]
#[path = "base_apply_tests.rs"]
mod base_apply_tests;
