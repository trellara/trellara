use super::*;

#[test]
fn target_failure_redelivers_before_commit() {
    let mut harness = Harness::new();

    let (failure, transaction) = inject_pre_commit_apply_failure(
        &mut harness.failure,
        FailurePoint::TargetFailureBeforeCommit,
        transaction("tx-1", 10),
        &mut harness.target,
        &mut harness.stream,
        &mut harness.steps,
    );

    assert_eq!(failure, ApplyFailure::RedeliveredBeforeCommit);
    assert!(transaction.is_none());
    assert_eq!(harness.target.applied_count(), 0);
    assert!(harness
        .steps
        .into_steps()
        .iter()
        .any(|step| { step.action == SimulationAction::TargetFailureBeforeCommit }));
}

#[test]
fn quarantine_repair_marks_replay_ready_before_redelivery() {
    let mut harness = Harness::new();

    let (failure, transaction) = inject_pre_commit_apply_failure(
        &mut harness.failure,
        FailurePoint::TargetQuarantineRepairReplay,
        transaction("tx-1", 10),
        &mut harness.target,
        &mut harness.stream,
        &mut harness.steps,
    );

    let steps = harness.steps.into_steps();
    let quarantine_index = steps
        .iter()
        .position(|step| step.action == SimulationAction::TargetQuarantined)
        .expect("quarantine step");
    let replay_ready_index = steps
        .iter()
        .position(|step| step.action == SimulationAction::OperatorMarkedReplayReady)
        .expect("replay-ready step");

    assert_eq!(failure, ApplyFailure::RedeliveredBeforeCommit);
    assert!(transaction.is_none());
    assert!(quarantine_index < replay_ready_index);
}

#[test]
fn unrelated_failure_returns_transaction_for_apply() {
    let mut harness = Harness::new();

    let (failure, transaction) = inject_pre_commit_apply_failure(
        &mut harness.failure,
        FailurePoint::StreamAckLossAfterApply,
        transaction("tx-1", 10),
        &mut harness.target,
        &mut harness.stream,
        &mut harness.steps,
    );

    assert_eq!(failure, ApplyFailure::None);
    assert_eq!(transaction.expect("transaction").id, "tx-1");
    assert!(harness.steps.into_steps().is_empty());
}

struct Harness {
    failure: FailureInjection,
    target: BaseTarget,
    stream: BaseStream,
    steps: BaseSimulationSteps,
}

impl Harness {
    fn new() -> Self {
        Self {
            failure: FailureInjection::default(),
            target: BaseTarget::default(),
            stream: BaseStream::default(),
            steps: BaseSimulationSteps::default(),
        }
    }
}

fn transaction(id: &str, lsn: u64) -> Transaction {
    Transaction {
        id: id.to_string(),
        lsn,
    }
}
