use crate::strict_chunk::{StrictChunkFailurePoint, StrictChunkSimulationAction};
use crate::strict_chunk_publish::{publish_all_chunks, publish_chunk, publish_chunks_except};
use crate::strict_chunk_state::StrictChunkSimState;
use crate::strict_chunk_target::{
    stage_after_manifest, target_can_apply, target_waits_for_manifest,
    target_waits_for_missing_chunk,
};

impl StrictChunkSimState {
    pub(crate) fn run(&mut self) {
        if self.should_run_manifest_before_chunks() {
            self.run_manifest_before_chunks();
            return;
        }

        if self.should_run_manifest_with_missing_chunk() {
            self.run_manifest_with_missing_chunk();
            return;
        }

        self.run_ordered_publish_path();
    }

    fn run_ordered_publish_path(&mut self) {
        self.publish_all_chunks(StrictChunkSimulationAction::ChunkPublished);

        if self.should_inject(StrictChunkFailurePoint::RelayCrashAfterChunksBeforeManifest) {
            self.mark_failure("relay crashed after durable chunk publish before manifest publish");
            self.push_step(
                None,
                StrictChunkSimulationAction::RelayCrashedBeforeManifest,
            );
            self.target_waits_for_manifest();
            self.publish_all_chunks(StrictChunkSimulationAction::ChunkReplayed);
        }

        self.manifest_published = true;
        self.push_step(None, StrictChunkSimulationAction::ManifestPublished);

        if self.should_inject(StrictChunkFailurePoint::RelayCrashAfterManifestBeforeSourceAck) {
            self.mark_failure("relay crashed after manifest publish before source feedback");
            self.push_step(
                None,
                StrictChunkSimulationAction::RelayCrashedBeforeSourceAck,
            );
            self.push_step(None, StrictChunkSimulationAction::ManifestReplayed);
        }

        self.ack_source();

        if self.should_inject(StrictChunkFailurePoint::TargetCrashAfterStagingBeforeCommit) {
            self.stage_after_manifest();
            self.mark_failure("target crashed after staging strict chunks before commit");
            self.push_step(None, StrictChunkSimulationAction::TargetCrashedBeforeCommit);
            self.push_step(
                None,
                StrictChunkSimulationAction::TargetDiscardedUncommittedStage,
            );
            self.push_step(None, StrictChunkSimulationAction::ManifestReplayed);
        }

        self.apply_after_manifest();
    }

    pub(crate) fn ack_source(&mut self) {
        self.source_acknowledged_lsn = Some(self.transaction.lsn);
        self.push_step(None, StrictChunkSimulationAction::SourceAcked);
    }

    pub(crate) fn publish_all_chunks(&mut self, action: StrictChunkSimulationAction) {
        publish_all_chunks(
            &mut self.chunks,
            &mut self.steps,
            self.config.chunk_count,
            action,
        );
    }

    pub(crate) fn publish_chunks_except(
        &mut self,
        except: u32,
        action: StrictChunkSimulationAction,
    ) {
        publish_chunks_except(
            &mut self.chunks,
            &mut self.steps,
            self.config.chunk_count,
            except,
            action,
        );
    }

    pub(crate) fn publish_chunk(&mut self, chunk_id: u32, action: StrictChunkSimulationAction) {
        publish_chunk(&mut self.chunks, &mut self.steps, chunk_id, action);
    }

    fn target_waits_for_manifest(&mut self) {
        target_waits_for_manifest(self.manifest_published, &mut self.steps);
    }

    pub(crate) fn target_waits_for_missing_chunk(&mut self) {
        target_waits_for_missing_chunk(
            self.manifest_published,
            &self.chunks,
            self.config.chunk_count,
            &mut self.steps,
        );
    }

    fn stage_after_manifest(&mut self) {
        stage_after_manifest(
            self.manifest_published,
            &self.chunks,
            self.config.chunk_count,
            &mut self.steps,
        );
    }

    pub(crate) fn apply_after_manifest(&mut self) {
        if target_can_apply(
            self.manifest_published,
            &self.chunks,
            self.config.chunk_count,
        ) {
            self.applied_transactions = 1;
            self.target_applied_lsn = Some(self.transaction.lsn);
            self.push_step(
                None,
                StrictChunkSimulationAction::TargetAppliedAfterManifest,
            );
        }
    }

    pub(crate) fn should_inject(&mut self, failure_point: StrictChunkFailurePoint) -> bool {
        self.failure
            .should_inject(self.config.failure_point, failure_point)
    }

    pub(crate) fn mark_failure(&mut self, failure: impl Into<String>) {
        self.failure.mark(failure);
    }

    pub(crate) fn push_step(&mut self, chunk_id: Option<u32>, action: StrictChunkSimulationAction) {
        self.steps.push(chunk_id, action);
    }
}
