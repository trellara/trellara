use super::*;

pub(super) fn assert_local_stream_contract(summary: &ChaosRunSummary) {
    assert_local_index_contract(summary);
    assert_local_publish_ack_contract(summary);
}

fn assert_local_index_contract(summary: &ChaosRunSummary) {
    let local_index = scenario(summary, "local_stream_index_rebuild");
    assert_eq!(
        local_index.invariant,
        "local_index_rebuild_preserves_replay_offsets"
    );
    assert_eq!(local_index.boundary_mode, "brokerless_local_stream");
    assert!(local_index
        .proof_command
        .contains("missing_index_is_rebuilt_for_offset_replay"));
    assert!(local_index
        .proof_command
        .contains("stale_index_discovers_durable_tail_without_truncating"));
    assert!(local_index
        .proof_command
        .contains("inspect_reports_corrupt_index_rebuild"));
    assert!(local_index
        .proof_command
        .contains("inspect_reports_torn_tail_bytes_before_recovery_append"));
    assert!(local_index
        .proof_command
        .contains("local_stream_inspect_summary_reports_depth_and_pending_messages"));
    assert!(local_index
        .expected_safety_property
        .contains("last valid replay offset"));
    assert!(local_index
        .evidence
        .contains("inspect_reports_torn_tail_bytes_before_recovery_append"));
    assert!(local_index
        .evidence
        .contains("stale_index_discovers_durable_tail_without_truncating"));
    assert!(local_index
        .evidence
        .contains("inspect_reports_corrupt_index_rebuild"));
    assert!(local_index
        .evidence
        .contains("local_stream_inspect_summary_reports_depth_and_pending_messages"));
    assert_eq!(
        local_index.recovery_command.as_deref(),
        Some("trellara stream inspect-local --config <flow>")
    );
}

fn assert_local_publish_ack_contract(summary: &ChaosRunSummary) {
    let local_ack = scenario(summary, "local_stream_publish_ack_after_fsync_durability");
    assert_eq!(
        local_ack.invariant,
        "source_ack_after_local_fsync_durability"
    );
    assert_eq!(local_ack.boundary_mode, "brokerless_local_stream");
    assert!(local_ack
        .proof_command
        .contains("fsync_publish_ack_follows_durable_frame_and_index"));
    assert!(local_ack
        .expected_safety_property
        .contains("zero-torn-tail inspection state"));
    assert_eq!(
        local_ack.recovery_command.as_deref(),
        Some("trellara stream inspect-local --config <flow>")
    );
}
