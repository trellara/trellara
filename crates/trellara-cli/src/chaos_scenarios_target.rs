use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn target_apply_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "applier_before_target_commit",
                    failure_point: "target apply fails before the transaction commits",
                    invariant: "target_checkpoint_same_transaction_as_apply",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property: "target checkpoint and dedup state are not advanced",
                    proof_command: "cargo test -p trellara-apply-postgres missing_target_table_fails_closed_without_checkpoint_or_dedup",
                    recovery_command: Some("trellara quarantine list --config <flow>"),
                    evidence: "trellara-apply-postgres::missing_target_table_fails_closed_without_checkpoint_or_dedup",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "applier_after_target_commit_before_stream_ack",
                    failure_point: "target commit succeeds but stream ack fails",
                    invariant: "transaction_dedup_before_reapply",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property:
                        "transaction-level dedup makes redelivery safe and the replay is acknowledged",
                    proof_command: "cargo test -p trellara-apply-postgres apply_worker_replays_safely_when_ack_fails_after_apply",
                    recovery_command: None,
                    evidence: "trellara-apply-postgres::apply_worker_replays_safely_when_ack_fails_after_apply",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "duplicate_transaction_replay",
                    failure_point: "the same transaction envelope is delivered more than once",
                    invariant: "transaction_dedup_before_reapply",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property:
                        "second delivery is skipped without reapplying target changes",
                    proof_command: "cargo test -p trellara-apply-postgres applies_transaction_once_and_records_checkpoint",
                    recovery_command: None,
                    evidence: "trellara-apply-postgres::applies_transaction_once_and_records_checkpoint",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "target_schema_missing",
                    failure_point: "target table required by a transaction does not exist",
                    invariant: "fail_closed_target_contract",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property:
                        "transaction is quarantined without advancing checkpoint or dedup state",
                    proof_command: "cargo test -p trellara-apply-postgres missing_target_table_fails_closed_without_checkpoint_or_dedup",
                    recovery_command: Some("trellara quarantine replay-ready --config <flow> --transaction-id <tx> --commit-lsn <lsn>"),
                    evidence: "trellara-apply-postgres::missing_target_table_fails_closed_without_checkpoint_or_dedup",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "target_update_delete_zero_rows",
                    failure_point:
                        "target UPDATE or DELETE key predicate matches no row during apply",
                    invariant: "no_silent_target_divergence",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property:
                        "transaction is quarantined as no_rows_matched without advancing checkpoint or dedup state",
                    proof_command:
                        "cargo test -p trellara-apply-postgres update_delete_zero_row_match_fails_closed",
                    recovery_command: Some(
                        "trellara verify --config <flow>; trellara reseed --config <flow>",
                    ),
                    evidence: "trellara-apply-postgres::update_delete_zero_row_match_fails_closed",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "filtered_reseed_preserves_out_of_scope_rows",
                    failure_point:
                        "operator reseeds a tenant, store, or region-scoped subset after checksum drift",
                    invariant: "row_filter_reseed_scope_is_preserved",
                    boundary_mode: "filtered_reseed_repair",
                    expected_safety_property:
                        "reseed deletes and replaces only rows matching the configured row_filter while preserving target-owned columns and out-of-scope target rows",
                    proof_command:
                        "cargo test -p trellara-verify --test postgres_snapshot_integration postgres_reseed_with_row_filter_replaces_only_matching_target_rows",
                    recovery_command: Some("trellara reseed --config <flow> --table <schema.table>"),
                    evidence:
                        "trellara-verify::postgres_reseed_with_row_filter_replaces_only_matching_target_rows",
                }),
    ]
}
