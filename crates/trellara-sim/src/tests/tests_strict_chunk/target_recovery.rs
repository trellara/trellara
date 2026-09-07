use super::*;

#[test]
fn strict_chunk_target_crash_after_staging_applies_once_after_replay() {
    let report = run_strict_chunk_simulation(StrictChunkSimulationConfig::new(
        456,
        StrictChunkFailurePoint::TargetCrashAfterStagingBeforeCommit,
    ));

    assert!(report.passed);
    assert_eq!(report.chunks_published, report.chunk_count);
    assert_eq!(report.duplicate_chunks, 0);
    assert_eq!(report.applied_transactions, 1);
    assert_eq!(report.target_applied_lsn, Some(report.commit_lsn));
    let staged_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetStagedAfterManifest)
        .expect("target stage step");
    let crash_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetCrashedBeforeCommit)
        .expect("target crash step");
    let discard_index = report
        .steps
        .iter()
        .position(|step| {
            step.action == StrictChunkSimulationAction::TargetDiscardedUncommittedStage
        })
        .expect("discard step");
    let replay_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::ManifestReplayed)
        .expect("manifest replay step");
    let apply_index = report
        .steps
        .iter()
        .position(|step| step.action == StrictChunkSimulationAction::TargetAppliedAfterManifest)
        .expect("apply step");
    assert!(staged_index < crash_index);
    assert!(crash_index < discard_index);
    assert!(discard_index < replay_index);
    assert!(replay_index < apply_index);
}
