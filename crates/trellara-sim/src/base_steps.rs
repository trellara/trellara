use crate::base::{SimulationAction, SimulationStep};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BaseSimulationSteps {
    steps: Vec<SimulationStep>,
}

impl BaseSimulationSteps {
    pub(crate) fn push(
        &mut self,
        transaction_id: String,
        commit_lsn: u64,
        action: SimulationAction,
    ) {
        self.steps.push(SimulationStep {
            transaction_id,
            commit_lsn,
            action,
        });
    }

    pub(crate) fn into_steps(self) -> Vec<SimulationStep> {
        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_ordered_transaction_steps() {
        let mut steps = BaseSimulationSteps::default();

        steps.push("tx-1".to_string(), 10, SimulationAction::Published);
        steps.push("tx-1".to_string(), 10, SimulationAction::StreamAcked);

        assert_eq!(
            steps.into_steps(),
            vec![
                SimulationStep {
                    transaction_id: "tx-1".to_string(),
                    commit_lsn: 10,
                    action: SimulationAction::Published,
                },
                SimulationStep {
                    transaction_id: "tx-1".to_string(),
                    commit_lsn: 10,
                    action: SimulationAction::StreamAcked,
                },
            ]
        );
    }
}
