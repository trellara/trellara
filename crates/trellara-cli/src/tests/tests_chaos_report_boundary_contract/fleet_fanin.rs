use super::*;

pub(super) fn assert_fleet_fanin_boundaries(summary: &ChaosRunSummary) {
    let fanin_gaps = scenario(summary, "fleet_fanin_offline_stores_publish_with_gaps");
    assert_eq!(fanin_gaps.invariant, "lake_epoch_gaps_are_explicit");
    assert_eq!(fanin_gaps.boundary_mode, "fleet_fanin_epoch_completeness");
    assert!(fanin_gaps
        .expected_safety_property
        .contains("complete_with_gaps"));

    let fanin_late = scenario(summary, "fleet_fanin_late_store_recovery");
    assert_eq!(
        fanin_late.invariant,
        "late_sources_recompute_epoch_completeness"
    );
    assert!(fanin_late.proof_command.contains("trellara-sim"));

    let fanin_duplicate = scenario(summary, "fleet_fanin_duplicate_store_replay");
    assert_eq!(
        fanin_duplicate.invariant,
        "fleet_fanin_deduplicates_by_transaction_boundary"
    );

    let fanin_conflict = scenario(summary, "fleet_fanin_conflicting_duplicate_quarantine");
    assert_eq!(
        fanin_conflict.invariant,
        "fleet_fanin_conflicting_duplicates_fail_closed"
    );
    assert!(fanin_conflict
        .expected_safety_property
        .contains("quarantines the epoch"));
}
