use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn snapshot_handoff_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_duplicate_copy_attempt",
                    failure_point:
                        "operator reruns the same snapshot run after all selected tables reached copy_complete",
                    invariant: "snapshot_copy_idempotent_by_run_and_table",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "completed table copies are reported as skipped instead of creating a second initial copy",
                    proof_command:
                        "cargo test -p trellara-sim snapshot_duplicate_copy_attempt_skips_completed_table_before_handoff && cargo test -p trellara-cli completed_snapshot_summary_reuses_handoff_ready_run",
                    recovery_command: None,
                    evidence: "trellara-sim::snapshot_duplicate_copy_attempt_skips_completed_table_before_handoff",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_incomplete_copy_resume",
                    failure_point:
                        "snapshot run is still copying a table when the process is restarted",
                    invariant: "snapshot_handoff_requires_complete_tables",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "active or incomplete table progress is not falsely treated as handoff-ready",
                    proof_command: "cargo test -p trellara-cli completed_snapshot_summary_requires_terminal_run_and_complete_tables",
                    recovery_command: Some("trellara snapshot --config <flow> --run-id <run>"),
                    evidence: "trellara-cli::completed_snapshot_summary_requires_terminal_run_and_complete_tables",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_source_crash_during_copy",
                    failure_point:
                        "source connection fails while exporting a table from the initial snapshot",
                    invariant: "snapshot_source_failure_retries_before_handoff",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "source copy failure marks the run recoverable, retries the table, and records handoff only after all selected tables are complete",
                    proof_command:
                        "cargo test -p trellara-sim snapshot_source_crash_during_table_copy_retries_before_handoff",
                    recovery_command: Some("trellara snapshot --config <flow> --run-id <run>"),
                    evidence:
                        "trellara-sim::snapshot_source_crash_during_table_copy_retries_before_handoff",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_relay_crash_during_copy",
                    failure_point:
                        "relay exits while copying a table from the exported source snapshot",
                    invariant: "snapshot_copy_idempotent_by_table",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "the table copy is retried and handoff is recorded only after every selected table is complete",
                    proof_command:
                        "cargo test -p trellara-sim snapshot_relay_crash_during_table_copy_retries_before_handoff",
                    recovery_command: Some("trellara snapshot --config <flow> --run-id <run>"),
                    evidence:
                        "trellara-sim::snapshot_relay_crash_during_table_copy_retries_before_handoff",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_target_crash_during_copy",
                    failure_point:
                        "target connection fails while receiving a table from the exported source snapshot",
                    invariant: "snapshot_copy_no_handoff_until_complete",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "target copy is retried and stream handoff is withheld until convergence proof can include all tables",
                    proof_command:
                        "cargo test -p trellara-sim snapshot_target_crash_during_table_copy_retries_before_handoff",
                    recovery_command: Some("trellara snapshot --config <flow> --run-id <run>"),
                    evidence:
                        "trellara-sim::snapshot_target_crash_during_table_copy_retries_before_handoff",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_copy_failure_marked_recoverable",
                    failure_point:
                        "table copy fails before all selected relations reach copy_complete",
                    invariant: "snapshot_failure_state_is_durable_before_retry",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "snapshot run and table progress are marked failed_recoverable before retry, and correctness reports surface the recovery action",
                    proof_command:
                        "cargo test -p trellara-sim snapshot_relay_crash_during_table_copy_retries_before_handoff && cargo test -p trellara-cli --lib snapshot_copy_failure_records_recoverable_run_and_table_state && cargo test -p trellara-cli --lib correctness_report_surfaces_recoverable_snapshot_copy_failure",
                    recovery_command: Some("trellara snapshot --config <flow> --run-id <run>"),
                    evidence:
                        "trellara-cli::snapshot_copy_failure_records_recoverable_run_and_table_state",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_invalid_state_regression",
                    failure_point:
                        "snapshot run attempts to move backward from copying or handoff into an earlier state",
                    invariant: "snapshot_state_machine_monotonic_until_recoverable",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "unsafe backward transitions are rejected unless the run is explicitly marked recoverable",
                    proof_command:
                        "cargo test -p trellara-checkpoint snapshot_run_state_rejects_unsafe_backward_transitions",
                    recovery_command: Some("trellara snapshot --config <flow> --run-id <run> --force"),
                    evidence: "trellara-checkpoint::snapshot_run_state_rejects_unsafe_backward_transitions",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "snapshot_exported_visibility_boundary",
                    failure_point:
                        "source receives writes after the logical slot exports the initial snapshot",
                    invariant: "snapshot_copy_uses_exported_lsn_boundary",
                    boundary_mode: "initial_snapshot_to_stream_handoff",
                    expected_safety_property:
                        "initial copy sees only rows visible in the exported snapshot; later writes must arrive through the stream",
                    proof_command:
                        "cargo test -p trellara-verify postgres_reseed_imports_exported_source_snapshot",
                    recovery_command: None,
                    evidence: "trellara-verify::postgres_reseed_imports_exported_source_snapshot",
                }),
    ]
}
