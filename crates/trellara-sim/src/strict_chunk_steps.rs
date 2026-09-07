use crate::strict_chunk::{StrictChunkSimulationAction, StrictChunkSimulationStep};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StrictChunkSimulationSteps {
    steps: Vec<StrictChunkSimulationStep>,
}

impl StrictChunkSimulationSteps {
    pub(crate) fn push(&mut self, chunk_id: Option<u32>, action: StrictChunkSimulationAction) {
        self.steps
            .push(StrictChunkSimulationStep { chunk_id, action });
    }

    pub(crate) fn into_steps(self) -> Vec<StrictChunkSimulationStep> {
        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_ordered_steps_with_optional_chunk_context() {
        let mut steps = StrictChunkSimulationSteps::default();

        steps.push(Some(2), StrictChunkSimulationAction::ChunkPublished);
        steps.push(None, StrictChunkSimulationAction::ManifestPublished);

        assert_eq!(
            steps.into_steps(),
            vec![
                StrictChunkSimulationStep {
                    chunk_id: Some(2),
                    action: StrictChunkSimulationAction::ChunkPublished,
                },
                StrictChunkSimulationStep {
                    chunk_id: None,
                    action: StrictChunkSimulationAction::ManifestPublished,
                },
            ]
        );
    }
}
