use super::*;

#[test]
fn checkpoint_failure_redelivers_without_advancing_apply_lsn() {
    let mut stream = stream_with(transaction("tx-1", 10));
    let mut target = BaseTarget::default();
    let mut failure = FailureInjection::default();
    let mut steps = BaseSimulationSteps::default();

    let applied_lsn = apply_next_message(
        &mut stream,
        &mut target,
        &mut failure,
        FailurePoint::CheckpointFailureDuringApply,
        &mut steps,
    );

    assert_eq!(applied_lsn, None);
    assert_eq!(target.applied_count(), 0);
    assert_eq!(stream.published_messages(), 1);
    assert!(steps
        .into_steps()
        .iter()
        .any(|step| { step.action == SimulationAction::CheckpointFailureRolledBackApply }));
}

#[test]
fn stream_ack_loss_after_apply_redelivers_for_dedup() {
    let mut stream = stream_with(transaction("tx-1", 10));
    let mut target = BaseTarget::default();
    let mut failure = FailureInjection::default();
    let mut steps = BaseSimulationSteps::default();

    let applied_lsn = apply_next_message(
        &mut stream,
        &mut target,
        &mut failure,
        FailurePoint::StreamAckLossAfterApply,
        &mut steps,
    );

    assert_eq!(applied_lsn, Some(10));
    assert!(target.has_applied("tx-1"));

    let duplicate_lsn = apply_next_message(
        &mut stream,
        &mut target,
        &mut failure,
        FailurePoint::StreamAckLossAfterApply,
        &mut steps,
    );

    assert_eq!(duplicate_lsn, None);
    assert_eq!(target.skipped_duplicate_count(), 1);
    assert_eq!(stream.acknowledged_messages(), 1);
}

#[test]
fn quarantine_repair_redelivers_before_apply() {
    let mut stream = stream_with(transaction("tx-1", 10));
    let mut target = BaseTarget::default();
    let mut failure = FailureInjection::default();
    let mut steps = BaseSimulationSteps::default();

    assert_eq!(
        apply_next_message(
            &mut stream,
            &mut target,
            &mut failure,
            FailurePoint::TargetQuarantineRepairReplay,
            &mut steps,
        ),
        None
    );
    assert_eq!(
        apply_next_message(
            &mut stream,
            &mut target,
            &mut failure,
            FailurePoint::TargetQuarantineRepairReplay,
            &mut steps,
        ),
        Some(10)
    );
    assert!(target.has_applied("tx-1"));
}

fn stream_with(transaction: Transaction) -> BaseStream {
    let mut stream = BaseStream::default();
    stream.publish(transaction);
    stream
}

fn transaction(id: &str, lsn: u64) -> Transaction {
    Transaction {
        id: id.to_string(),
        lsn,
    }
}
