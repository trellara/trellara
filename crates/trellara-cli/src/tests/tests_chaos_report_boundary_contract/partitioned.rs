use super::*;

pub(super) fn assert_partitioned_barrier_boundaries(summary: &ChaosRunSummary) {
    let partition_manifest = scenario(summary, "partition_manifest_missing_chunk");
    assert_eq!(
        partition_manifest.invariant,
        "manifest_barrier_before_partition_apply"
    );
    assert_eq!(partition_manifest.boundary_mode, "partitioned_scale_mode");
    assert_eq!(
        partition_manifest.recovery_command.as_deref(),
        Some("trellara partition-watermarks --config <flow>")
    );
    assert!(partition_manifest
        .proof_command
        .contains("barrier_reconstruction_rejects_missing_chunks"));
    assert!(partition_manifest
        .proof_command
        .contains("barrier_reconstruction_rejects_exact_duplicate_chunks"));
    assert!(partition_manifest
        .proof_command
        .contains("barrier_reconstruction_rejects_duplicate_manifest_partitions"));
    assert!(partition_manifest
        .proof_command
        .contains("commit_marker_rejects_duplicate_manifest_partitions"));
    assert!(partition_manifest
        .expected_safety_property
        .contains("malformed manifest partition boundaries"));

    let routing_headers = scenario(summary, "partition_chunk_routing_header_mismatch");
    assert_eq!(
        routing_headers.invariant,
        "partition_chunk_headers_are_apply_trust_boundary"
    );
    assert!(routing_headers
        .proof_command
        .contains("chunk_message_rejects_envelope_transaction_mismatch"));
    assert!(routing_headers
        .proof_command
        .contains("barrier_worker_rejects_chunk_partition_routing_header_mismatch"));
    assert!(routing_headers
        .proof_command
        .contains("barrier_worker_rejects_unlisted_chunk_buffered_before_manifest"));
    assert!(routing_headers
        .proof_command
        .contains("barrier_worker_rejects_unlisted_chunk_after_manifest"));
    assert!(routing_headers
        .proof_command
        .contains("pending_stats_reports_extra_chunks_separately"));
    assert!(routing_headers
        .expected_safety_property
        .contains("extra chunk blockers"));

    let partition_commit_marker = scenario(summary, "partition_commit_marker_ack_ambiguous");
    assert_eq!(
        partition_commit_marker.invariant,
        "partition_source_checkpoint_waits_for_commit_marker_ack"
    );
    assert_eq!(
        partition_commit_marker.boundary_mode,
        "partitioned_scale_mode"
    );
    assert!(partition_commit_marker
        .proof_command
        .contains("partitioned_ambiguous_commit_marker_publish"));

    let marker_mismatch = scenario(summary, "partition_commit_marker_manifest_mismatch");
    assert_eq!(
        marker_mismatch.invariant,
        "partition_commit_marker_matches_manifest_before_apply"
    );
    assert_eq!(marker_mismatch.boundary_mode, "partitioned_scale_mode");
    assert!(marker_mismatch
        .proof_command
        .contains("manifest_message_rejects_envelope_boundary_mismatch"));
    assert!(marker_mismatch
        .proof_command
        .contains("commit_marker_message_rejects_envelope_boundary_mismatch"));
    assert!(marker_mismatch
        .proof_command
        .contains("barrier_worker_rejects_mismatched_commit_marker"));
    assert!(marker_mismatch
        .proof_command
        .contains("pending_stats_reports_invalid_commit_marker_separately"));
    assert!(marker_mismatch
        .expected_safety_property
        .contains("before target apply or acknowledgement"));

    let partition_before_ack = scenario(summary, "partition_applier_before_ack");
    assert_eq!(
        partition_before_ack.recovery_command.as_deref(),
        Some("trellara status --config <flow> --view dashboard --format text")
    );
}
