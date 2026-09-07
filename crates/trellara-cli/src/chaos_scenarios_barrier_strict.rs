use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn strict_chunk_barrier_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "strict_chunk_manifest_missing_chunk",
            failure_point: "a strict chunked transaction manifest arrives without every referenced chunk",
            invariant: "strict_chunk_manifest_barrier_before_apply",
            boundary_mode: "strict_chunked_transaction_order",
            expected_safety_property:
                "barrier reconstruction rejects incomplete large transactions before target apply",
            proof_command:
                "cargo test -p trellara-sim strict_chunk_missing_chunk_waits_for_replay_before_apply",
            recovery_command: Some("trellara status --config <flow> --view report --format text"),
            evidence: "trellara-sim::strict_chunk_missing_chunk_waits_for_replay_before_apply",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "strict_chunk_out_of_order_arrival",
            failure_point: "a strict chunked transaction manifest arrives before its chunk messages",
            invariant: "strict_chunk_manifest_waits_for_completeness",
            boundary_mode: "strict_chunked_transaction_order",
            expected_safety_property:
                "barrier-aware apply buffers the manifest and waits for every chunk before applying",
            proof_command:
                "cargo test -p trellara-sim strict_chunk_manifest_before_chunks_waits_for_complete_chunk_set",
            recovery_command: Some("trellara apply --config <flow>"),
            evidence:
                "trellara-sim::strict_chunk_manifest_before_chunks_waits_for_complete_chunk_set",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "strict_chunk_relay_crash_after_chunks_before_manifest",
            failure_point:
                "relay exits after every strict chunk is durably published but before the manifest and commit marker barrier is published",
            invariant: "strict_chunk_manifest_required_before_apply",
            boundary_mode: "strict_chunked_transaction_order",
            expected_safety_property:
                "target waits for the manifest and commit marker barrier, replayed chunks are deduplicated, and no partial transaction is applied",
            proof_command:
                "cargo test -p trellara-sim strict_chunk_crash_after_chunks_waits_for_manifest_before_apply",
            recovery_command: Some("trellara relay --config <flow>"),
            evidence:
                "trellara-sim::strict_chunk_crash_after_chunks_waits_for_manifest_before_apply",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "strict_chunk_partial_publish_failure",
            failure_point:
                "relay fails while publishing a multi-message strict chunked transaction before every barrier message is acknowledged",
            invariant: "source_checkpoint_waits_for_all_barrier_messages",
            boundary_mode: "strict_chunked_transaction_order",
            expected_safety_property:
                "source durable checkpoint and source feedback do not advance until every chunk, manifest, and commit marker publish succeeds",
            proof_command:
                "cargo test -p trellara-relay strict_chunked_partial_publish_failure_does_not_advance_checkpoint_or_source_ack",
            recovery_command: Some("trellara relay --config <flow>"),
            evidence:
                "trellara-relay::strict_chunked_partial_publish_failure_does_not_advance_checkpoint_or_source_ack",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "strict_chunk_relay_crash_after_manifest_before_source_ack",
            failure_point:
                "relay exits after the strict chunk manifest is durably published but before source feedback",
            invariant: "strict_chunk_source_ack_after_manifest_publish",
            boundary_mode: "strict_chunked_transaction_order",
            expected_safety_property:
                "source checkpoint remains behind until the complete chunk set and manifest can be replayed safely",
            proof_command:
                "cargo test -p trellara-sim strict_chunk_crash_after_manifest_recovers_without_moving_source_ack_early",
            recovery_command: Some("trellara relay --config <flow>"),
            evidence:
                "trellara-sim::strict_chunk_crash_after_manifest_recovers_without_moving_source_ack_early",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "strict_chunk_target_crash_after_staging_before_commit",
            failure_point:
                "target stages a complete strict chunked transaction then crashes before commit and checkpoint",
            invariant: "strict_chunk_stage_not_visible_before_target_commit",
            boundary_mode: "strict_chunked_transaction_order",
            expected_safety_property:
                "uncommitted staged chunks are discarded on restart, the manifest is replayed, and the target applies the transaction exactly once",
            proof_command:
                "cargo test -p trellara-sim strict_chunk_target_crash_after_staging_applies_once_after_replay",
            recovery_command: Some("trellara apply --config <flow>"),
            evidence:
                "trellara-sim::strict_chunk_target_crash_after_staging_applies_once_after_replay",
        }),
    ]
}
