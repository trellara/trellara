use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn partitioned_barrier_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "partition_manifest_missing_chunk",
            failure_point: "a partitioned transaction manifest arrives without every referenced chunk",
            invariant: "manifest_barrier_before_partition_apply",
            boundary_mode: "partitioned_scale_mode",
            expected_safety_property:
                "barrier reconstruction rejects incomplete, duplicate, or malformed manifest partition boundaries before target apply",
            proof_command: "cargo test -p trellara-protocol barrier_reconstruction_rejects_missing_chunks && cargo test -p trellara-protocol barrier_reconstruction_rejects_exact_duplicate_chunks && cargo test -p trellara-protocol barrier_reconstruction_rejects_duplicate_manifest_partitions && cargo test -p trellara-protocol commit_marker_rejects_duplicate_manifest_partitions",
            recovery_command: Some("trellara partition-watermarks --config <flow>"),
            evidence: "trellara-protocol::barrier_reconstruction_rejects_missing_chunks; trellara-protocol::barrier_reconstruction_rejects_exact_duplicate_chunks; trellara-protocol::barrier_reconstruction_rejects_duplicate_manifest_partitions; trellara-protocol::commit_marker_rejects_duplicate_manifest_partitions",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "partition_chunk_routing_header_mismatch",
            failure_point:
                "a partition chunk is delivered with routing headers, payload identity, or manifest membership that disagrees with the decoded transaction boundary",
            invariant: "partition_chunk_headers_are_apply_trust_boundary",
            boundary_mode: "partitioned_scale_mode",
            expected_safety_property:
                "stream producers and target appliers reject mismatched partition id, event count, checksum, transaction identity, or unlisted manifest membership before buffering, applying, or acknowledging the chunk, and runtime pending stats expose extra chunk blockers",
            proof_command:
                "cargo test -p trellara-stream chunk_message_rejects_envelope_transaction_mismatch --lib && cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_chunk_partition_routing_header_mismatch && cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_unlisted_chunk_buffered_before_manifest && cargo test -p trellara-apply-postgres --lib barrier_worker_rejects_unlisted_chunk_after_manifest && cargo test -p trellara-apply-postgres pending_stats_reports_extra_chunks_separately && cargo test -p trellara-stream-local --lib reconstruct_local_barrier_transaction_rejects_chunk_header_payload_mismatch",
            recovery_command: Some("trellara run --local --verify --config <flow>"),
            evidence:
                "trellara-stream::chunk_message_rejects_envelope_transaction_mismatch; trellara-apply-postgres::barrier_worker_rejects_chunk_partition_routing_header_mismatch; trellara-apply-postgres::barrier_worker_rejects_unlisted_chunk_buffered_before_manifest; trellara-apply-postgres::barrier_worker_rejects_unlisted_chunk_after_manifest; trellara-apply-postgres::pending_stats_reports_extra_chunks_separately; trellara-stream-local::reconstruct_local_barrier_transaction_rejects_chunk_header_payload_mismatch",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "partition_commit_marker_ack_ambiguous",
            failure_point:
                "relay publishes the partitioned manifest and commit marker but the commit-marker acknowledgement is ambiguous",
            invariant: "partition_source_checkpoint_waits_for_commit_marker_ack",
            boundary_mode: "partitioned_scale_mode",
            expected_safety_property:
                "source durable checkpoint and source feedback do not advance until the partition chunk, manifest, and commit marker are all acknowledged",
            proof_command:
                "cargo test -p trellara-relay partitioned_ambiguous_commit_marker_publish_does_not_advance_checkpoint",
            recovery_command: Some("trellara relay --config <flow>"),
            evidence:
                "trellara-relay::partitioned_ambiguous_commit_marker_publish_does_not_advance_checkpoint",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "partition_commit_marker_manifest_mismatch",
            failure_point:
                "a partitioned commit marker arrives with a payload that does not match the transaction manifest",
            invariant: "partition_commit_marker_matches_manifest_before_apply",
            boundary_mode: "partitioned_scale_mode",
            expected_safety_property:
                "stream producers reject mismatched barrier payload identity; the barrier-aware applier rejects mismatched commit markers before target apply or acknowledgement, and runtime pending stats expose invalid commit-marker blockers",
            proof_command:
                "cargo test -p trellara-stream manifest_message_rejects_envelope_boundary_mismatch --lib && cargo test -p trellara-stream commit_marker_message_rejects_envelope_boundary_mismatch --lib && cargo test -p trellara-apply-postgres barrier_worker_rejects_mismatched_commit_marker && cargo test -p trellara-apply-postgres pending_stats_reports_invalid_commit_marker_separately && cargo test -p trellara-cli apply_summary_exposes_invalid_commit_marker_blocker",
            recovery_command: Some("trellara status --config <flow> --view report --format text"),
            evidence:
                "trellara-stream::manifest_message_rejects_envelope_boundary_mismatch; trellara-stream::commit_marker_message_rejects_envelope_boundary_mismatch; trellara-apply-postgres::barrier_worker_rejects_mismatched_commit_marker; trellara-apply-postgres::pending_stats_reports_invalid_commit_marker_separately; trellara-cli::apply_summary_exposes_invalid_commit_marker_blocker",
        }),
    ]
}
