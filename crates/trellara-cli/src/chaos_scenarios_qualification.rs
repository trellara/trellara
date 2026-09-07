use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn qualification_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "qualification_source_promotion_while_relay_disconnected",
            failure_point: "source primary is promoted while the relay is disconnected from the old primary",
            invariant: "promoted_source_replays_from_last_durable_ack",
            boundary_mode: "source_failover_slot_to_relay_restart",
            expected_safety_property:
                "relay restart on the promoted source replays from the last durable acknowledgement, downstream dedup absorbs the replay, and source acknowledgement lag remains visible while disconnected",
            proof_command:
                "cargo test -p trellara-sim qualification_source_promotion_while_relay_disconnected",
            recovery_command: Some(
                "trellara check --config <flow> --format text; trellara relay --config <flow>",
            ),
            evidence:
                "trellara-sim::qualification_source_promotion_while_relay_disconnected_recovers_via_failover_slot",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "qualification_broker_outage_quorum_loss",
            failure_point: "broker outage removes publish quorum while the relay is processing source transactions",
            invariant: "source_ack_waits_for_broker_quorum",
            boundary_mode: "broker_quorum_publish_ack",
            expected_safety_property:
                "source feedback is withheld during quorum loss, publish retry is observable, and acknowledgement advances only after broker quorum returns",
            proof_command:
                "cargo test -p trellara-sim qualification_broker_outage_quorum_loss",
            recovery_command: Some(
                "trellara status --config <flow> --view alerts --format text; trellara relay --config <flow>",
            ),
            evidence:
                "trellara-sim::qualification_broker_outage_quorum_loss_withholds_source_ack",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "qualification_target_restart_during_apply",
            failure_point: "target Postgres or the applier process restarts after apply begins but before the target transaction commits",
            invariant: "target_restart_replays_uncheckpointed_apply",
            boundary_mode: "target_apply_transaction_commit",
            expected_safety_property:
                "uncheckpointed apply state rolls back, redelivery applies the transaction exactly once, and diagnostics name the replay boundary",
            proof_command:
                "cargo test -p trellara-sim qualification_target_restart_during_apply",
            recovery_command: Some(
                "trellara status --config <flow> --view diagnostics --format text; trellara apply --config <flow>",
            ),
            evidence:
                "trellara-sim::qualification_target_restart_during_apply_replays_once",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "qualification_object_store_success_catalog_timeout",
            failure_point:
                "raw CDC files are durably written to object storage but the Iceberg catalog commit times out",
            invariant: "catalog_timeout_cannot_publish_uncommitted_epoch",
            boundary_mode: "object_store_write_before_catalog_visibility",
            expected_safety_property:
                "the epoch remains pending_catalog_commit, catalog retry discovers the existing object-store receipt, and downstream Spark consumption is held until catalog evidence is durable",
            proof_command:
                "cargo test -p trellara-sim qualification_object_store_success_catalog_timeout",
            recovery_command: Some("trellara lake fanin verify --config <flow> --format json"),
            evidence:
                "trellara-sim::qualification_object_store_success_catalog_timeout_retries_catalog_before_visibility",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "qualification_twenty_four_hour_soak_large_transaction_memory_ceiling",
            failure_point:
                "24-hour soak includes large streamed pgoutput transactions that must spill before relay memory grows without bound",
            invariant: "soak_large_transactions_stay_within_memory_ceiling",
            boundary_mode: "large_transaction_stream_spill_memory_ceiling",
            expected_safety_property:
                "the qualification harness records a 24-hour soak window, proves large transactions spill before publish, and asserts peak relay memory stays below the configured ceiling",
            proof_command:
                "cargo test -p trellara-sim qualification_twenty_four_hour_soak_large_transaction_memory_ceiling",
            recovery_command: Some(
                "trellara performance --config <flow> --format json; trellara chaos report --output docs/correctness-report.html",
            ),
            evidence:
                "trellara-sim::qualification_twenty_four_hour_soak_large_transaction_memory_ceiling",
        }),
    ]
}
