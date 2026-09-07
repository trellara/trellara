use super::*;

pub(super) fn assert_repair_replay_boundaries(summary: &ChaosRunSummary) {
    let repair_replay = scenario(summary, "repair_and_replay_ready");
    assert_eq!(repair_replay.status, ChaosScenarioStatus::CoveredByTests);
    assert!(repair_replay.proof_command.contains("trellara-sim"));

    let replay_boundary = scenario(summary, "quarantine_replay_ready_requires_exact_boundary");
    assert_eq!(
        replay_boundary.invariant,
        "replay_ready_requires_exact_quarantine_boundary"
    );
    assert!(replay_boundary
        .proof_command
        .contains("quarantine_replay_ready_refuses_unknown_transaction_boundary"));
    assert!(replay_boundary
        .expected_safety_property
        .contains("exact quarantined transaction boundary"));
}
