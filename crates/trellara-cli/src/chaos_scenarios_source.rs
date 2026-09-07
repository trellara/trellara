use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn source_and_stream_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "relay_before_broker_publish",
                    failure_point: "relay exits before publishing a captured source transaction",
                    invariant: "source_checkpoint_after_broker_ack",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property:
                        "source durable checkpoint is not advanced, so the transaction is captured again",
                    proof_command: "cargo test -p trellara-relay failed_publish_does_not_advance_checkpoint",
                    recovery_command: None,
                    evidence: "trellara-relay::failed_publish_does_not_advance_checkpoint",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "relay_after_broker_publish_before_source_feedback",
                    failure_point: "relay publishes successfully but exits before source durable feedback",
                    invariant: "source_checkpoint_after_broker_ack",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property:
                        "duplicate stream delivery is possible, but no committed transaction is missed",
                    proof_command: "cargo test -p trellara-relay ambiguous_publish_replays_duplicate_without_advancing_checkpoint",
                    recovery_command: None,
                    evidence: "trellara-relay::ambiguous_publish_replays_duplicate_without_advancing_checkpoint",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "source_failover_after_publish_before_ack",
                    failure_point:
                        "source primary is promoted after durable stream publish but before old-source feedback",
                    invariant: "failover_slot_replay_preserves_transaction_boundary",
                    boundary_mode: "strict_transaction_order_with_failover_slot",
                    expected_safety_property:
                        "relay restart on the promoted source may redeliver the last transaction, but downstream dedup preserves exactly-once apply",
                    proof_command: "cargo test -p trellara-sim source_failover_after_publish_before_ack_recovers_with_duplicate_replay",
                    recovery_command: Some("trellara check --config <flow>"),
                    evidence: "trellara-sim::source_failover_after_publish_before_ack_recovers_with_duplicate_replay",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "source_failover_slot_unsynced",
                    failure_point:
                        "source failover slot is configured but not synchronized before standby promotion",
                    invariant: "failover_slot_sync_before_promotion",
                    boundary_mode: "source_failover_readiness",
                    expected_safety_property:
                        "source-safety warns before operators rely on a promoted standby that may not retain the CDC boundary",
                    proof_command:
                        "cargo test -p trellara-cli --lib source_safety_warns_when_failover_slot_is_not_synced",
                    recovery_command: Some("trellara check --config <flow> --format text"),
                    evidence: "trellara-cli::source_safety_warns_when_failover_slot_is_not_synced",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "source_failover_slot_disabled",
                    failure_point:
                        "source replication slot is not configured as a failover slot before promotion",
                    invariant: "failover_slot_enabled_before_promotion",
                    boundary_mode: "source_failover_readiness",
                    expected_safety_property:
                        "source-safety reports the disabled failover posture as an explicit recovery boundary before CDC depends on promotion",
                    proof_command:
                        "cargo test -p trellara-cli --lib source_safety_warns_when_failover_slot_is_disabled && cargo test -p trellara-cli --lib direct_source_safety_warns_when_failover_slot_is_disabled",
                    recovery_command: Some("trellara check --config <flow> --format text"),
                    evidence: "trellara-cli::source_safety_warns_when_failover_slot_is_disabled; trellara-cli::direct_source_safety_warns_when_failover_slot_is_disabled",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "source_slot_abandoned_idle_timeout",
                    failure_point:
                        "a logical replication slot is inactive long enough to threaten WAL retention or depend on idle slot cleanup",
                    invariant: "inactive_slot_cleanup_posture_is_explicit",
                    boundary_mode: "source_safety_slot_lifecycle",
                    expected_safety_property:
                        "source-safety degrades the flow, names inactive-since metadata, and distinguishes disabled from configured idle_replication_slot_timeout cleanup",
                    proof_command:
                        "cargo test -p trellara-cli --lib direct_source_safety_mentions_disabled_idle_slot_timeout && cargo test -p trellara-cli --lib direct_source_safety_surfaces_configured_idle_slot_timeout",
                    recovery_command: Some("trellara-check <source> --format text"),
                    evidence:
                        "trellara-cli::direct_source_safety_mentions_disabled_idle_slot_timeout; trellara-cli::direct_source_safety_surfaces_configured_idle_slot_timeout",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "broker_publish_failure",
                    failure_point: "stream publisher returns an error while relay is processing a transaction",
                    invariant: "source_checkpoint_after_broker_ack",
                    boundary_mode: "strict_transaction_order",
                    expected_safety_property:
                        "relay does not advance source checkpoint without broker acknowledgement",
                    proof_command: "cargo test -p trellara-relay failed_publish_does_not_advance_checkpoint",
                    recovery_command: None,
                    evidence: "trellara-relay::failed_publish_does_not_advance_checkpoint",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "local_stream_index_rebuild",
                    failure_point:
                        "brokerless local replay index is missing, stale, or corrupt after process restart",
                    invariant: "local_index_rebuild_preserves_replay_offsets",
                    boundary_mode: "brokerless_local_stream",
                    expected_safety_property:
                        "the local stream rebuilds the derived offset index from durable log frames, excludes torn tails, and reports the last valid replay offset plus locate/seek commands",
                    proof_command: "cargo test -p trellara-stream-local missing_index_is_rebuilt_for_offset_replay --lib && cargo test -p trellara-stream-local stale_index_discovers_durable_tail_without_truncating --lib && cargo test -p trellara-stream-local inspect_reports_corrupt_index_rebuild --lib && cargo test -p trellara-stream-local inspect_reports_torn_tail_bytes_before_recovery_append --lib && cargo test -p trellara-cli local_stream_inspect_summary_reports_depth_and_pending_messages --lib",
                    recovery_command: Some("trellara stream inspect-local --config <flow>"),
                    evidence: "trellara-stream-local::missing_index_is_rebuilt_for_offset_replay; trellara-stream-local::stale_index_discovers_durable_tail_without_truncating; trellara-stream-local::inspect_reports_corrupt_index_rebuild; trellara-stream-local::inspect_reports_torn_tail_bytes_before_recovery_append; trellara-cli::local_stream_inspect_summary_reports_depth_and_pending_messages",
                }),
                ChaosScenarioSummary::covered(ChaosScenarioInput {
                    name: "local_stream_publish_ack_after_fsync_durability",
                    failure_point:
                        "brokerless local publish returns acknowledgement before durable frame and index evidence is replayable",
                    invariant: "source_ack_after_local_fsync_durability",
                    boundary_mode: "brokerless_local_stream",
                    expected_safety_property:
                        "a returned publish acknowledgement identifies an offset whose frame, sidecar index entry, and zero-torn-tail inspection state are immediately replayable under default fsync durability",
                    proof_command:
                        "cargo test -p trellara-stream-local fsync_publish_ack_follows_durable_frame_and_index --lib",
                    recovery_command: Some("trellara stream inspect-local --config <flow>"),
                    evidence: "trellara-stream-local::fsync_publish_ack_follows_durable_frame_and_index",
                }),
    ]
}
