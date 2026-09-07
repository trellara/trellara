use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(super) fn pgoutput_identity_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "missing_replica_identity",
            failure_point:
                "source table has no primary key, identity index, or full row replica identity for updates/deletes",
            invariant: "preflight_before_cdc_start",
            boundary_mode: "strict_transaction_order",
            expected_safety_property: "preflight and contract-test fail before relay starts",
            proof_command:
                "cargo test -p trellara-pg-capture capture_preflight_reports_replica_identity_safety",
            recovery_command: Some("trellara contract-test --config <flow>"),
            evidence: "trellara-pg-capture::capture_preflight_reports_replica_identity_safety",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "default_replica_identity_primary_key_apply",
            failure_point:
                "source table uses REPLICA IDENTITY DEFAULT with a stable primary key instead of FULL row identity",
            invariant: "primary_key_predicate_apply_without_full",
            boundary_mode: "strict_transaction_order",
            expected_safety_property:
                "UPDATE and DELETE statements use key predicates, so ordinary primary-key tables do not require REPLICA IDENTITY FULL",
            proof_command: "cargo test -p trellara-apply-postgres plans_update_with_key_predicate",
            recovery_command: Some("trellara contract-test --config <flow>"),
            evidence: "trellara-apply-postgres::plans_update_with_key_predicate",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "default_replica_identity_key_change_apply",
            failure_point:
                "source UPDATE changes a primary-key value while pgoutput provides the old key boundary",
            invariant: "primary_key_moves_match_old_key_and_set_new_key",
            boundary_mode: "strict_transaction_order",
            expected_safety_property:
                "UPDATE statements match the old key from the before image and assign the new key from the after image in the same target transaction",
            proof_command:
                "cargo test -p trellara-apply-postgres key_changing_update_sets_new_key_and_matches_old_key && cargo test -p trellara-apply-postgres --test postgres_integration key_changing_update_moves_primary_key_with_old_key_predicate",
            recovery_command: Some("trellara contract-test --config <flow>"),
            evidence: "trellara-apply-postgres::key_changing_update_sets_new_key_and_matches_old_key",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "unchanged_toast_columns_preserved",
            failure_point:
                "pgoutput omits unchanged non-key TOAST values from an UPDATE under default replica identity",
            invariant: "absent_toast_columns_are_unchanged",
            boundary_mode: "pgoutput_relation_metadata",
            expected_safety_property:
                "apply plans omit absent or explicit unchanged TOAST markers for non-key columns so target data is preserved",
            proof_command:
                "cargo test -p trellara-apply-postgres update_omits_absent_non_key_columns_for_unchanged_toast && cargo test -p trellara-apply-postgres update_omits_explicit_unchanged_toast_marker",
            recovery_command: Some("trellara contract-test --config <flow>"),
            evidence:
                "trellara-apply-postgres::update_omits_absent_non_key_columns_for_unchanged_toast; trellara-apply-postgres::update_omits_explicit_unchanged_toast_marker",
        }),
    ]
}
