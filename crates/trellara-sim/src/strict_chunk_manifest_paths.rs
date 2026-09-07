use crate::strict_chunk::{
    StrictChunkFailurePoint, StrictChunkSimulationAction, StrictChunkSimulationConfig,
};
use crate::strict_chunk_state::StrictChunkSimState;

impl StrictChunkSimState {
    pub(crate) fn run_manifest_before_chunks(&mut self) {
        self.mark_failure(
            "strict chunk manifest arrived before chunk messages; target waited for complete chunk set",
        );
        self.manifest_published = true;
        self.push_step(
            None,
            StrictChunkSimulationAction::ManifestArrivedBeforeChunks,
        );
        self.target_waits_for_missing_chunk();
        self.publish_all_chunks(StrictChunkSimulationAction::ChunkReplayed);
        self.ack_source();
        self.apply_after_manifest();
    }

    pub(crate) fn run_manifest_with_missing_chunk(&mut self) {
        let withheld_chunk = last_chunk_id(&self.config);
        self.publish_chunks_except(withheld_chunk, StrictChunkSimulationAction::ChunkPublished);
        self.push_step(
            Some(withheld_chunk),
            StrictChunkSimulationAction::ChunkWithheld,
        );
        self.manifest_published = true;
        self.push_step(None, StrictChunkSimulationAction::ManifestPublished);
        self.mark_failure(
            "strict chunk manifest referenced a chunk that was not yet durable; target withheld visibility until replay filled the gap",
        );
        self.target_waits_for_missing_chunk();
        self.publish_chunk(
            withheld_chunk,
            StrictChunkSimulationAction::MissingChunkReplayed,
        );
        self.ack_source();
        self.apply_after_manifest();
    }

    pub(crate) fn should_run_manifest_before_chunks(&mut self) -> bool {
        self.should_inject(StrictChunkFailurePoint::ManifestArrivesBeforeChunks)
    }

    pub(crate) fn should_run_manifest_with_missing_chunk(&mut self) -> bool {
        self.should_inject(StrictChunkFailurePoint::ManifestArrivesWithMissingChunk)
    }
}

fn last_chunk_id(config: &StrictChunkSimulationConfig) -> u32 {
    u32::try_from(config.chunk_count.saturating_sub(1)).expect("strict chunk count fits in u32")
}
