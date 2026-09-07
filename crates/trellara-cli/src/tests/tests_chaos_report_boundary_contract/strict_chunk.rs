use super::*;

pub(super) fn assert_strict_chunk_boundaries(summary: &ChaosRunSummary) {
    let strict_missing = scenario(summary, "strict_chunk_manifest_missing_chunk");
    assert_eq!(strict_missing.status, ChaosScenarioStatus::CoveredByTests);
    assert_eq!(
        strict_missing.boundary_mode,
        "strict_chunked_transaction_order"
    );
    assert_eq!(
        strict_missing.recovery_command.as_deref(),
        Some("trellara status --config <flow> --view report --format text")
    );
    assert!(strict_missing
        .proof_command
        .contains("strict_chunk_missing_chunk_waits_for_replay_before_apply"));

    let strict_out_of_order = scenario(summary, "strict_chunk_out_of_order_arrival");
    assert_eq!(
        strict_out_of_order.status,
        ChaosScenarioStatus::CoveredByTests
    );
    assert!(strict_out_of_order
        .proof_command
        .contains("strict_chunk_manifest_before_chunks_waits_for_complete_chunk_set"));

    let strict_crash_before_manifest = scenario(
        summary,
        "strict_chunk_relay_crash_after_chunks_before_manifest",
    );
    assert_eq!(
        strict_crash_before_manifest.invariant,
        "strict_chunk_manifest_required_before_apply"
    );
    assert!(strict_crash_before_manifest
        .proof_command
        .contains("strict_chunk_crash_after_chunks_waits_for_manifest_before_apply"));

    let strict_crash_before_ack = scenario(
        summary,
        "strict_chunk_relay_crash_after_manifest_before_source_ack",
    );
    assert_eq!(
        strict_crash_before_ack.invariant,
        "strict_chunk_source_ack_after_manifest_publish"
    );

    let strict_target_crash = scenario(
        summary,
        "strict_chunk_target_crash_after_staging_before_commit",
    );
    assert_eq!(
        strict_target_crash.invariant,
        "strict_chunk_stage_not_visible_before_target_commit"
    );
    assert!(strict_target_crash
        .proof_command
        .contains("strict_chunk_target_crash_after_staging_applies_once_after_replay"));

    let strict_partial_publish = scenario(summary, "strict_chunk_partial_publish_failure");
    assert_eq!(
        strict_partial_publish.invariant,
        "source_checkpoint_waits_for_all_barrier_messages"
    );
    assert!(strict_partial_publish
        .proof_command
        .contains("strict_chunked_partial_publish_failure"));

    let property = scenario(summary, "protocol_property_chunk_manifest_reconstruction");
    assert_eq!(property.status, ChaosScenarioStatus::CoveredByTests);
    assert_eq!(
        property.boundary_mode,
        "strict_chunked_or_partitioned_barrier"
    );
    assert!(property
        .proof_command
        .contains("strict_chunk_manifest_property_reconstructs_source_order"));
    assert!(property
        .proof_command
        .contains("partitioned_manifest_property_reconstructs_source_order"));

    let strict_checksum = scenario(summary, "strict_chunk_checksum_tampering");
    assert_eq!(
        strict_checksum.invariant,
        "chunk_checksum_mismatch_fails_closed"
    );
    assert_eq!(
        strict_checksum.boundary_mode,
        "strict_chunked_transaction_order"
    );
    assert!(strict_checksum
        .proof_command
        .contains("barrier_reconstruction_rejects_chunk_checksum_tampering"));

    let barrier_conflict = scenario(summary, "barrier_conflicting_duplicate_messages");
    assert_eq!(
        barrier_conflict.invariant,
        "conflicting_duplicate_barrier_messages_fail_closed"
    );
    assert_eq!(
        barrier_conflict.boundary_mode,
        "strict_chunked_or_partitioned_barrier"
    );
    assert!(barrier_conflict
        .proof_command
        .contains("barrier_worker_rejects_conflicting_duplicate_chunks"));
    assert!(barrier_conflict
        .proof_command
        .contains("barrier_worker_rejects_conflicting_duplicate_manifests"));
    assert!(barrier_conflict
        .proof_command
        .contains("barrier_worker_rejects_conflicting_duplicate_commit_markers_before_manifest"));
    assert!(barrier_conflict
        .expected_safety_property
        .contains("before target apply or stream acknowledgement"));
}
