use super::*;

pub(super) fn assert_snapshot_handoff_boundaries(summary: &ChaosRunSummary) {
    let snapshot_duplicate = scenario(summary, "snapshot_duplicate_copy_attempt");
    assert_eq!(
        snapshot_duplicate.invariant,
        "snapshot_copy_idempotent_by_run_and_table"
    );
    assert_eq!(
        snapshot_duplicate.boundary_mode,
        "initial_snapshot_to_stream_handoff"
    );
    assert!(snapshot_duplicate.proof_command.contains("trellara-sim"));

    let snapshot_source_crash = scenario(summary, "snapshot_source_crash_during_copy");
    assert_eq!(
        snapshot_source_crash.invariant,
        "snapshot_source_failure_retries_before_handoff"
    );
    assert_eq!(
        snapshot_source_crash.boundary_mode,
        "initial_snapshot_to_stream_handoff"
    );
    assert!(snapshot_source_crash.proof_command.contains("trellara-sim"));

    let snapshot_handoff = scenario(summary, "snapshot_handoff_recorded_before_stream_start");
    assert_eq!(snapshot_handoff.status, ChaosScenarioStatus::CoveredByTests);
    assert!(snapshot_handoff.proof_command.contains("trellara-sim"));

    let snapshot_relay_crash = scenario(summary, "snapshot_relay_crash_during_copy");
    assert_eq!(
        snapshot_relay_crash.invariant,
        "snapshot_copy_idempotent_by_table"
    );

    let snapshot_target_crash = scenario(summary, "snapshot_target_crash_during_copy");
    assert_eq!(
        snapshot_target_crash.invariant,
        "snapshot_copy_no_handoff_until_complete"
    );

    let snapshot_ddl = scenario(summary, "snapshot_ddl_during_copy");
    assert_eq!(snapshot_ddl.status, ChaosScenarioStatus::CoveredByTests);
    assert_eq!(snapshot_ddl.invariant, "snapshot_contract_before_handoff");
    assert!(snapshot_ddl.proof_command.contains("trellara-sim"));
}
