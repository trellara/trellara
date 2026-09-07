use super::*;

pub(super) fn assert_replica_identity_contract(summary: &ChaosRunSummary) {
    let default_identity = scenario(summary, "default_replica_identity_primary_key_apply");
    assert_eq!(
        default_identity.invariant,
        "primary_key_predicate_apply_without_full"
    );
    assert!(default_identity
        .expected_safety_property
        .contains("do not require REPLICA IDENTITY FULL"));
    assert!(default_identity
        .proof_command
        .contains("plans_update_with_key_predicate"));

    let key_change = scenario(summary, "default_replica_identity_key_change_apply");
    assert_eq!(
        key_change.invariant,
        "primary_key_moves_match_old_key_and_set_new_key"
    );
    assert!(key_change
        .proof_command
        .contains("key_changing_update_sets_new_key_and_matches_old_key"));

    let toast = scenario(summary, "unchanged_toast_columns_preserved");
    assert_eq!(toast.invariant, "absent_toast_columns_are_unchanged");
    assert!(toast
        .proof_command
        .contains("update_omits_absent_non_key_columns_for_unchanged_toast"));
    assert!(toast
        .proof_command
        .contains("update_omits_explicit_unchanged_toast_marker"));

    let zero_row_apply = scenario(summary, "target_update_delete_zero_rows");
    assert_eq!(zero_row_apply.invariant, "no_silent_target_divergence");
    assert!(zero_row_apply
        .expected_safety_property
        .contains("no_rows_matched"));
    assert!(zero_row_apply
        .proof_command
        .contains("update_delete_zero_row_match_fails_closed"));
}
