use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn fleet_fanin_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "fleet_fanin_offline_stores_publish_with_gaps",
                    failure_point:
                        "some required store sources are offline when a lake epoch reaches its sealing boundary",
                    invariant: "lake_epoch_gaps_are_explicit",
                    boundary_mode: "fleet_fanin_epoch_completeness",
                    expected_safety_property:
                        "a publish-with-gaps policy marks missing sources explicitly and only exposes the epoch as complete_with_gaps",
                    proof_command:
                        "cargo test -p trellara-sim fleet_fanin_offline_stores_publish_with_explicit_gap_state",
                    recovery_command: Some("trellara lake plan --config <flow> --format json"),
                    evidence:
                        "trellara-sim::fleet_fanin_offline_stores_publish_with_explicit_gap_state",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "fleet_fanin_late_store_recovery",
                    failure_point:
                        "offline store sources arrive after an epoch was published with explicit gaps",
                    invariant: "late_sources_recompute_epoch_completeness",
                    boundary_mode: "fleet_fanin_epoch_completeness",
                    expected_safety_property:
                        "late source envelopes can recompute the same epoch from complete_with_gaps to complete without double counting prior stores",
                    proof_command:
                        "cargo test -p trellara-sim fleet_fanin_late_sources_recompute_epoch_to_complete",
                    recovery_command: Some("trellara lake plan --config <flow> --format json"),
                    evidence:
                        "trellara-sim::fleet_fanin_late_sources_recompute_epoch_to_complete",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "fleet_fanin_duplicate_store_replay",
                    failure_point:
                        "store CDC replay redelivers already-counted source transactions into the fan-in epoch",
                    invariant: "fleet_fanin_deduplicates_by_transaction_boundary",
                    boundary_mode: "fleet_fanin_epoch_completeness",
                    expected_safety_property:
                        "identical source transaction replays are skipped so epoch transaction and change counts stay stable",
                    proof_command:
                        "cargo test -p trellara-sim fleet_fanin_duplicate_store_replay_is_deduplicated",
                    recovery_command: None,
                    evidence:
                        "trellara-sim::fleet_fanin_duplicate_store_replay_is_deduplicated",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "fleet_fanin_conflicting_duplicate_quarantine",
                    failure_point:
                        "a replayed store event uses an existing idempotency key with different transaction evidence",
                    invariant: "fleet_fanin_conflicting_duplicates_fail_closed",
                    boundary_mode: "fleet_fanin_epoch_completeness",
                    expected_safety_property:
                        "conflicting duplicate evidence quarantines the epoch instead of publishing an ambiguous current-state or SCD2 view",
                    proof_command:
                        "cargo test -p trellara-sim fleet_fanin_conflicting_duplicate_quarantines_epoch",
                    recovery_command: Some("trellara status --config <flow> --view diagnostics --format text"),
                    evidence:
                        "trellara-sim::fleet_fanin_conflicting_duplicate_quarantines_epoch",
                }),
    ]
}
