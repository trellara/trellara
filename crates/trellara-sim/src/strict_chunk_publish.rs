use crate::strict_chunk::StrictChunkSimulationAction;
use crate::strict_chunk_steps::StrictChunkSimulationSteps;
use crate::strict_chunk_tracker::StrictChunkTracker;

pub(crate) fn publish_all_chunks(
    chunks: &mut StrictChunkTracker,
    steps: &mut StrictChunkSimulationSteps,
    chunk_count: usize,
    action: StrictChunkSimulationAction,
) {
    for chunk_id in chunk_ids(chunk_count) {
        publish_chunk(chunks, steps, chunk_id, action.clone());
    }
}

pub(crate) fn publish_chunks_except(
    chunks: &mut StrictChunkTracker,
    steps: &mut StrictChunkSimulationSteps,
    chunk_count: usize,
    except: u32,
    action: StrictChunkSimulationAction,
) {
    for chunk_id in chunk_ids(chunk_count) {
        if chunk_id != except {
            publish_chunk(chunks, steps, chunk_id, action.clone());
        }
    }
}

pub(crate) fn publish_chunk(
    chunks: &mut StrictChunkTracker,
    steps: &mut StrictChunkSimulationSteps,
    chunk_id: u32,
    action: StrictChunkSimulationAction,
) {
    chunks.publish(chunk_id);
    steps.push(Some(chunk_id), action);
}

fn chunk_ids(chunk_count: usize) -> impl Iterator<Item = u32> {
    let chunk_count = u32::try_from(chunk_count).expect("strict chunk count fits in u32");
    0..chunk_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_all_chunks_records_every_chunk_once() {
        let mut chunks = StrictChunkTracker::default();
        let mut steps = StrictChunkSimulationSteps::default();

        publish_all_chunks(
            &mut chunks,
            &mut steps,
            3,
            StrictChunkSimulationAction::ChunkPublished,
        );

        assert!(chunks.is_complete(3));
        assert_eq!(chunks.published_count(), 3);
        assert_eq!(chunks.duplicate_count(), 0);
        assert_eq!(steps.into_steps().len(), 3);
    }

    #[test]
    fn publish_chunks_except_leaves_gap_for_replay() {
        let mut chunks = StrictChunkTracker::default();
        let mut steps = StrictChunkSimulationSteps::default();

        publish_chunks_except(
            &mut chunks,
            &mut steps,
            4,
            2,
            StrictChunkSimulationAction::ChunkPublished,
        );

        assert_eq!(chunks.published_count(), 3);
        assert!(!chunks.is_complete(4));
        assert!(steps
            .into_steps()
            .iter()
            .all(|step| step.chunk_id != Some(2)));
    }

    #[test]
    #[should_panic(expected = "strict chunk count fits in u32")]
    fn publish_all_chunks_rejects_unrepresentable_chunk_count() {
        let mut chunks = StrictChunkTracker::default();
        let mut steps = StrictChunkSimulationSteps::default();

        publish_all_chunks(
            &mut chunks,
            &mut steps,
            usize::try_from(u64::from(u32::MAX) + 1).expect("usize can represent overflow case"),
            StrictChunkSimulationAction::ChunkPublished,
        );
    }
}
