use crate::fleet_fanin::{FleetFanInSimulationAction, FleetFanInSimulationStep};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FleetFanInSteps {
    steps: Vec<FleetFanInSimulationStep>,
}

impl FleetFanInSteps {
    pub(crate) fn push(&mut self, source_id: Option<String>, action: FleetFanInSimulationAction) {
        self.steps
            .push(FleetFanInSimulationStep { source_id, action });
    }

    pub(crate) fn into_steps(self) -> Vec<FleetFanInSimulationStep> {
        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_ordered_steps_with_optional_source_context() {
        let mut steps = FleetFanInSteps::default();

        steps.push(
            Some("store-0001".to_string()),
            FleetFanInSimulationAction::SourceEnvelopePublished,
        );
        steps.push(None, FleetFanInSimulationAction::EpochPublishedWithGaps);

        assert_eq!(
            steps.into_steps(),
            vec![
                FleetFanInSimulationStep {
                    source_id: Some("store-0001".to_string()),
                    action: FleetFanInSimulationAction::SourceEnvelopePublished,
                },
                FleetFanInSimulationStep {
                    source_id: None,
                    action: FleetFanInSimulationAction::EpochPublishedWithGaps,
                },
            ]
        );
    }
}
