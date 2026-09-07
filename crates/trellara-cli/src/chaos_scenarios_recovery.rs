use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn late_recovery_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_ddl_during_copy",
                    failure_point: "source schema changes while a table is being copied",
                    invariant: "snapshot_contract_before_handoff",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "schema drift withholds handoff until contract refresh and table-copy retry complete",
                    proof_command:
                        "cargo test -p trellara-sim snapshot_ddl_during_table_copy_withholds_handoff_until_contract_refresh",
                    recovery_command: Some("trellara snapshot --config <flow> --run-id <run> --force"),
                    evidence:
                        "trellara-sim::snapshot_ddl_during_table_copy_withholds_handoff_until_contract_refresh",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_handoff_recorded_before_stream_start",
                    failure_point:
                        "snapshot copy reaches stream_handoff_ready but relay has not yet started",
                    invariant: "snapshot_handoff_before_stream_checkpoint",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "post-snapshot writes are replayed only after the stream restarts from the recorded handoff boundary",
                    proof_command:
                        "cargo test -p trellara-sim handoff_recorded_before_stream_start_recovers_before_replay",
                    recovery_command: Some("trellara relay --config <flow>"),
                    evidence:
                        "trellara-sim::handoff_recorded_before_stream_start_recovers_before_replay",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "repair_and_replay_ready",
                    failure_point: "operator repairs a quarantined target contract",
                    invariant: "operator_replay_preserves_transaction_boundary",
                    boundary_mode: "strict_transaction_order_or_partitioned_scale_mode",
                    expected_safety_property:
                        "dedup/quarantine state can be cleared so redelivery applies instead of skipping",
                    proof_command:
                        "cargo test -p trellara-sim target_quarantine_repair_replay_applies_after_operator_marks_replay_ready",
                    recovery_command: Some("trellara quarantine replay-ready --config <flow> --transaction-id <tx> --commit-lsn <lsn>"),
                    evidence:
                        "trellara-sim::target_quarantine_repair_replay_applies_after_operator_marks_replay_ready",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "quarantine_replay_ready_requires_exact_boundary",
                    failure_point:
                        "operator marks a transaction replay-ready with the wrong transaction id or commit LSN",
                    invariant: "replay_ready_requires_exact_quarantine_boundary",
                    boundary_mode: "strict_transaction_order_or_partitioned_scale_mode",
                    expected_safety_property:
                        "replay-ready fails closed unless the exact quarantined transaction boundary exists, so operators do not clear dedup or request redelivery for the wrong CDC transaction",
                    proof_command:
                        "cargo test -p trellara-cli quarantine_replay_ready_refuses_unknown_transaction_boundary",
                    recovery_command: Some("trellara quarantine list --config <flow>"),
                    evidence: "trellara-cli::quarantine_replay_ready_refuses_unknown_transaction_boundary",
                }),
    ]
}
