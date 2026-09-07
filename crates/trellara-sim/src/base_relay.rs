use crate::base::{FailurePoint, SimulationAction};
use crate::base_steps::BaseSimulationSteps;
use crate::failure_injection::FailureInjection;
use crate::transaction::Transaction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RelayFailure {
    None,
    StopBeforeDurableCheckpoint,
    RecoverableDuplicate,
}

pub(crate) fn inject_relay_failure(
    failure: &mut FailureInjection,
    configured_failure: FailurePoint,
    transaction: &Transaction,
    steps: &mut BaseSimulationSteps,
) -> RelayFailure {
    if failure.should_inject(configured_failure, FailurePoint::PublishAckLoss) {
        failure.mark("publisher accepted the message but relay lost the acknowledgement");
        push_step(transaction, steps, SimulationAction::PublishAckLost);
        return RelayFailure::StopBeforeDurableCheckpoint;
    }
    if failure.should_inject(configured_failure, FailurePoint::DuplicateDelivery) {
        failure.mark("transport delivered the same transaction more than once");
        push_step(transaction, steps, SimulationAction::DuplicateDelivered);
        return RelayFailure::RecoverableDuplicate;
    }
    RelayFailure::None
}

fn push_step(transaction: &Transaction, steps: &mut BaseSimulationSteps, action: SimulationAction) {
    steps.push(transaction.id.clone(), transaction.lsn, action);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_ack_loss_stops_before_durable_checkpoint() {
        let mut failure = FailureInjection::default();
        let mut steps = BaseSimulationSteps::default();

        let result = inject_relay_failure(
            &mut failure,
            FailurePoint::PublishAckLoss,
            &transaction("tx-1", 10),
            &mut steps,
        );

        assert_eq!(result, RelayFailure::StopBeforeDurableCheckpoint);
        assert_eq!(
            steps.into_steps()[0].action,
            SimulationAction::PublishAckLost
        );
        assert!(failure
            .into_description()
            .is_some_and(|description| description.contains("lost the acknowledgement")));
    }

    #[test]
    fn duplicate_delivery_is_recoverable_duplicate() {
        let mut failure = FailureInjection::default();
        let mut steps = BaseSimulationSteps::default();

        let result = inject_relay_failure(
            &mut failure,
            FailurePoint::DuplicateDelivery,
            &transaction("tx-1", 10),
            &mut steps,
        );

        assert_eq!(result, RelayFailure::RecoverableDuplicate);
        assert_eq!(
            steps.into_steps()[0].action,
            SimulationAction::DuplicateDelivered
        );
    }

    #[test]
    fn unrelated_failure_point_does_not_inject_relay_failure() {
        let mut failure = FailureInjection::default();
        let mut steps = BaseSimulationSteps::default();

        let result = inject_relay_failure(
            &mut failure,
            FailurePoint::TargetFailureBeforeCommit,
            &transaction("tx-1", 10),
            &mut steps,
        );

        assert_eq!(result, RelayFailure::None);
        assert!(steps.into_steps().is_empty());
    }

    fn transaction(id: &str, lsn: u64) -> Transaction {
        Transaction {
            id: id.to_string(),
            lsn,
        }
    }
}
