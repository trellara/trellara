use crate::strict_chunk::StrictChunkSimulationAction;
use crate::strict_chunk_steps::StrictChunkSimulationSteps;
use crate::strict_chunk_tracker::StrictChunkTracker;

pub(crate) fn target_waits_for_manifest(
    manifest_published: bool,
    steps: &mut StrictChunkSimulationSteps,
) {
    if !manifest_published {
        steps.push(None, StrictChunkSimulationAction::TargetWaitedForManifest);
    }
}

pub(crate) fn target_waits_for_missing_chunk(
    manifest_published: bool,
    chunks: &StrictChunkTracker,
    chunk_count: usize,
    steps: &mut StrictChunkSimulationSteps,
) {
    if manifest_published && !chunks.is_complete(chunk_count) {
        steps.push(
            None,
            StrictChunkSimulationAction::TargetWaitedForMissingChunk,
        );
    }
}

pub(crate) fn stage_after_manifest(
    manifest_published: bool,
    chunks: &StrictChunkTracker,
    chunk_count: usize,
    steps: &mut StrictChunkSimulationSteps,
) {
    if target_can_apply(manifest_published, chunks, chunk_count) {
        steps.push(None, StrictChunkSimulationAction::TargetStagedAfterManifest);
    }
}

pub(crate) fn target_can_apply(
    manifest_published: bool,
    chunks: &StrictChunkTracker,
    chunk_count: usize,
) -> bool {
    manifest_published && chunks.is_complete(chunk_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_waits_for_manifest_until_manifest_arrives() {
        let mut steps = StrictChunkSimulationSteps::default();

        target_waits_for_manifest(false, &mut steps);
        target_waits_for_manifest(true, &mut steps);

        assert_eq!(steps.into_steps().len(), 1);
    }

    #[test]
    fn target_waits_for_missing_chunks_only_after_manifest() {
        let mut chunks = StrictChunkTracker::default();
        chunks.publish(0);
        let mut steps = StrictChunkSimulationSteps::default();

        target_waits_for_missing_chunk(false, &chunks, 2, &mut steps);
        target_waits_for_missing_chunk(true, &chunks, 2, &mut steps);

        assert_eq!(steps.into_steps().len(), 1);
    }

    #[test]
    fn target_can_apply_only_after_manifest_and_complete_chunk_set() {
        let mut chunks = StrictChunkTracker::default();
        chunks.publish(0);

        assert!(!target_can_apply(false, &chunks, 2));
        assert!(!target_can_apply(true, &chunks, 2));

        chunks.publish(1);

        assert!(target_can_apply(true, &chunks, 2));
    }
}
