use std::path::Path;

pub(crate) fn partitioned_scale_proof_command(repository_root: &Path) -> String {
    let config = repository_root
        .join("examples/retail-fleet/partitioned.yml")
        .display()
        .to_string();
    let commands = [
        format!("trellara partition-watermarks --config {config} --format text"),
        format!("trellara partition-rebalance-plan --config {config} --format json"),
        "cargo test -p trellara-checkpoint partition_watermark_rejects_cross_flow_checkpoint_evidence".to_string(),
        "cargo test -p trellara-checkpoint partition_visibility_ddl_ack --lib".to_string(),
        "cargo test -p trellara-protocol barrier_reconstruction_rejects_missing_chunks".to_string(),
        "cargo test -p trellara-protocol barrier_reconstruction_rejects_exact_duplicate_chunks".to_string(),
        "cargo test -p trellara-protocol barrier_reconstruction_rejects_duplicate_manifest_partitions".to_string(),
        "cargo test -p trellara-protocol commit_marker_rejects_duplicate_manifest_partitions".to_string(),
        "cargo test -p trellara-protocol partition_visibility --lib".to_string(),
        "cargo test -p trellara-protocol partitioned_scale_readiness --lib".to_string(),
        "cargo test -p trellara-stream strict_message_contains_envelope_headers_and_key --lib".to_string(),
        "cargo test -p trellara-stream envelope_headers_include_ddl_event_count_and_release_gates --lib".to_string(),
        "cargo test -p trellara-stream manifest_message_uses_barrier_topic_and_headers --lib".to_string(),
        "cargo test -p trellara-stream manifest_message_rejects_envelope_boundary_mismatch --lib".to_string(),
        "cargo test -p trellara-stream chunk_message_rejects_envelope_transaction_mismatch --lib".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_manifest_evidence_header_mismatch".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_manifest_missing_evidence_header".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_chunk_partitioned_scale_readiness_header_mismatch".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_requires_partitioned_scale_readiness_headers".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_chunk_partition_routing_header_mismatch".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_unlisted_chunk_buffered_before_manifest".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_unlisted_chunk_after_manifest".to_string(),
        "cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_corrupt_chunk_before_apply_or_ack".to_string(),
        "cargo test -p trellara-stream-local --lib reconstruct_local_barrier_transaction_rejects_chunk_header_payload_mismatch".to_string(),
        "cargo test -p trellara-stream-local --lib reconstruct_local_barrier_transaction_rejects_corrupt_chunk_manifest_proof".to_string(),
        "cargo test -p trellara-stream-local --lib reconstruct_local_barrier_transaction_rejects_duplicate_partition_offsets".to_string(),
        "cargo test -p trellara-relay partitioned_ambiguous_commit_marker_publish_does_not_advance_checkpoint".to_string(),
        "cargo test -p trellara-apply-postgres barrier_worker_rejects_mismatched_commit_marker".to_string(),
        "cargo test -p trellara-apply-postgres pending_stats_reports_invalid_commit_marker_separately".to_string(),
        "cargo test -p trellara-apply-postgres pending_stats_reports_extra_chunks_separately".to_string(),
        "cargo test -p trellara-cli apply_summary_exposes_invalid_commit_marker_blocker".to_string(),
        "cargo test -p trellara-apply-postgres manifest_envelope_records_partition_checkpoints_atomically".to_string(),
        "cargo test -p trellara-sim target_quarantine_repair_replay_applies_after_operator_marks_replay_ready".to_string(),
        "cargo test -p trellara-cli quarantine_replay_ready_refuses_unknown_transaction_boundary".to_string(),
        "cargo test -p trellara-cli quarantine_replay_contract_preserves_boundary_evidence".to_string(),
        "cargo test -p trellara-cli partitioned_mode_explains_transaction_barrier".to_string(),
        "cargo test -p trellara-cli tests_transaction_inspect --lib".to_string(),
        "cargo test -p trellara-cli tests_correctness_report --lib".to_string(),
    ];

    commands.join(" && ")
}
