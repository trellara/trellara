use super::*;

#[test]
fn strict_chunk_crash_after_chunks_waits_for_manifest_before_apply() {
    let report = run_strict_chunk_simulation(StrictChunkSimulationConfig::new(
        99,
        StrictChunkFailurePoint::RelayCrashAfterChunksBeforeManifest,
    ));

    assert!(report.passed);
    assert_eq!(report.chunks_published, report.chunk_count);
    assert_eq!(report.duplicate_chunks, report.chunk_count);
    assert!(report.manifest_published);
    assert_eq!(report.applied_transactions, 1);
    let wait_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetWaitedForManifest)
        .expect("wait step");
    let manifest_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::ManifestPublished)
        .expect("manifest step");
    let apply_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetAppliedAfterManifest)
        .expect("apply step");
    assert!(wait_index < manifest_index);
    assert!(manifest_index < apply_index);
}

#[test]
fn strict_chunk_crash_after_manifest_recovers_without_moving_source_ack_early() {
    let report = run_strict_chunk_simulation(StrictChunkSimulationConfig::new(
        123,
        StrictChunkFailurePoint::RelayCrashAfterManifestBeforeSourceAck,
    ));

    assert!(report.passed);
    assert_eq!(report.chunks_published, report.chunk_count);
    assert_eq!(report.duplicate_chunks, 0);
    assert_eq!(report.source_acknowledged_lsn, Some(report.commit_lsn));
    assert_eq!(report.target_applied_lsn, Some(report.commit_lsn));
    let crash_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::RelayCrashedBeforeSourceAck)
        .expect("crash step");
    let ack_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::SourceAcked)
        .expect("ack step");
    assert!(crash_index < ack_index);
}

#[test]
fn strict_chunk_missing_chunk_waits_for_replay_before_apply() {
    let report = run_strict_chunk_simulation(StrictChunkSimulationConfig::new(
        234,
        StrictChunkFailurePoint::ManifestArrivesWithMissingChunk,
    ));

    assert!(report.passed);
    assert_eq!(report.chunks_published, report.chunk_count);
    assert_eq!(report.duplicate_chunks, 0);
    assert_eq!(report.applied_transactions, 1);
    assert_eq!(report.target_applied_lsn, Some(report.commit_lsn));
    let withheld_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::ChunkWithheld)
        .expect("withheld chunk step");
    let manifest_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::ManifestPublished)
        .expect("manifest step");
    let wait_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetWaitedForMissingChunk)
        .expect("wait for missing chunk step");
    let replay_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::MissingChunkReplayed)
        .expect("missing chunk replay step");
    let apply_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetAppliedAfterManifest)
        .expect("apply step");
    assert!(withheld_index < manifest_index);
    assert!(manifest_index < wait_index);
    assert!(wait_index < replay_index);
    assert!(replay_index < apply_index);
}

#[test]
fn strict_chunk_manifest_before_chunks_waits_for_complete_chunk_set() {
    let report = run_strict_chunk_simulation(StrictChunkSimulationConfig::new(
        345,
        StrictChunkFailurePoint::ManifestArrivesBeforeChunks,
    ));

    assert!(report.passed);
    assert_eq!(report.chunks_published, report.chunk_count);
    assert_eq!(report.duplicate_chunks, 0);
    assert_eq!(report.applied_transactions, 1);
    assert_eq!(report.source_acknowledged_lsn, Some(report.commit_lsn));
    let manifest_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::ManifestArrivedBeforeChunks)
        .expect("manifest-before-chunks step");
    let wait_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetWaitedForMissingChunk)
        .expect("wait for chunks step");
    let first_chunk_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::ChunkReplayed)
        .expect("chunk replay step");
    let apply_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetAppliedAfterManifest)
        .expect("apply step");
    assert!(manifest_index < wait_index);
    assert!(wait_index < first_chunk_index);
    assert!(first_chunk_index < apply_index);
}
