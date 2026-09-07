use crate::ChaosRunSummary;

pub(crate) const MVP_SNAPSHOT_SCENARIOS: &[&str] = &[
    "snapshot_duplicate_copy_attempt",
    "snapshot_incomplete_copy_resume",
    "snapshot_source_crash_during_copy",
    "snapshot_relay_crash_during_copy",
    "snapshot_target_crash_during_copy",
    "snapshot_copy_failure_marked_recoverable",
    "snapshot_invalid_state_regression",
    "snapshot_exported_visibility_boundary",
    "snapshot_ddl_during_copy",
    "snapshot_handoff_recorded_before_stream_start",
];

pub(crate) const MVP_STRICT_CHUNK_SCENARIOS: &[&str] = &[
    "pgoutput_streamed_transaction_spills_until_commit",
    "strict_chunk_manifest_missing_chunk",
    "strict_chunk_out_of_order_arrival",
    "strict_chunk_relay_crash_after_chunks_before_manifest",
    "strict_chunk_partial_publish_failure",
    "strict_chunk_relay_crash_after_manifest_before_source_ack",
    "strict_chunk_target_crash_after_staging_before_commit",
    "protocol_property_chunk_manifest_reconstruction",
];

pub(crate) const MVP_PARTITIONED_SCALE_SCENARIOS: &[&str] = &[
    "partition_manifest_missing_chunk",
    "partition_chunk_routing_header_mismatch",
    "partition_commit_marker_ack_ambiguous",
    "partition_commit_marker_manifest_mismatch",
    "partition_applier_before_ack",
    "repair_and_replay_ready",
    "quarantine_replay_ready_requires_exact_boundary",
];

pub(crate) const MVP_SOURCE_FAILOVER_SCENARIOS: &[&str] = &[
    "source_failover_after_publish_before_ack",
    "source_failover_slot_unsynced",
    "source_failover_slot_disabled",
];

pub(crate) const MVP_SCHEMA_CHANGE_SCENARIOS: &[&str] = &[
    "pgoutput_schema_change_during_stream",
    "source_schema_handoff_recovery",
    "snapshot_ddl_during_copy",
];

pub(crate) fn chaos_has_scenario(summary: &ChaosRunSummary, name: &str) -> bool {
    summary
        .scenarios
        .iter()
        .any(|scenario| scenario.name == name)
}

pub(crate) fn count_chaos_scenarios(summary: &ChaosRunSummary, names: &[&str]) -> usize {
    names
        .iter()
        .filter(|name| chaos_has_scenario(summary, name))
        .count()
}
