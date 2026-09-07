use crate::snapshot::{SnapshotSimulationAction, SnapshotSimulationStep};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SnapshotSimulationSteps {
    steps: Vec<SnapshotSimulationStep>,
}

impl SnapshotSimulationSteps {
    pub(crate) fn push(&mut self, relation: Option<String>, action: SnapshotSimulationAction) {
        self.steps.push(SnapshotSimulationStep { relation, action });
    }

    pub(crate) fn into_steps(self) -> Vec<SnapshotSimulationStep> {
        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_ordered_steps_with_optional_relation_context() {
        let mut steps = SnapshotSimulationSteps::default();

        steps.push(
            Some("public.sales".to_string()),
            SnapshotSimulationAction::TableCopyStarted,
        );
        steps.push(None, SnapshotSimulationAction::HandoffRecorded);

        assert_eq!(
            steps.into_steps(),
            vec![
                SnapshotSimulationStep {
                    relation: Some("public.sales".to_string()),
                    action: SnapshotSimulationAction::TableCopyStarted,
                },
                SnapshotSimulationStep {
                    relation: None,
                    action: SnapshotSimulationAction::HandoffRecorded,
                },
            ]
        );
    }
}
