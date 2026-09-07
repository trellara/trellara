use super::*;

pub(super) fn assert_reseed_contract(summary: &ChaosRunSummary) {
    let filtered_reseed = scenario(summary, "filtered_reseed_preserves_out_of_scope_rows");
    assert_eq!(
        filtered_reseed.invariant,
        "row_filter_reseed_scope_is_preserved"
    );
    assert_eq!(filtered_reseed.boundary_mode, "filtered_reseed_repair");
    assert!(filtered_reseed
        .proof_command
        .contains("postgres_reseed_with_row_filter_replaces_only_matching_target_rows"));
    assert!(filtered_reseed
        .expected_safety_property
        .contains("out-of-scope target rows"));
    assert_eq!(
        filtered_reseed.recovery_command.as_deref(),
        Some("trellara reseed --config <flow> --table <schema.table>")
    );
}
