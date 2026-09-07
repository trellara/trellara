use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn barrier_integrity_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "protocol_property_chunk_manifest_reconstruction",
            failure_point:
                "generated strict chunk and partitioned manifests vary transaction size, chunk size, and partition fan-out",
            invariant: "manifest_barriers_reconstruct_source_order_for_all_generated_inputs",
            boundary_mode: "strict_chunked_or_partitioned_barrier",
            expected_safety_property:
                "property tests reconstruct every generated strict chunk or partitioned transaction in source order and reject missing chunks with minimized failing repros",
            proof_command:
                "cargo test -p trellara-protocol strict_chunk_manifest_property_reconstructs_source_order && cargo test -p trellara-protocol partitioned_manifest_property_reconstructs_source_order",
            recovery_command: None,
            evidence:
                "trellara-protocol::strict_chunk_manifest_property_reconstructs_source_order; trellara-protocol::partitioned_manifest_property_reconstructs_source_order",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "strict_chunk_checksum_tampering",
            failure_point:
                "a strict or partition chunk payload is altered after the manifest records its checksum",
            invariant: "chunk_checksum_mismatch_fails_closed",
            boundary_mode: "strict_chunked_transaction_order",
            expected_safety_property:
                "barrier reconstruction rejects the transaction before apply when a chunk checksum no longer matches the manifest",
            proof_command:
                "cargo test -p trellara-protocol barrier_reconstruction_rejects_chunk_checksum_tampering",
            recovery_command: Some("trellara status --config <flow> --view report --format text"),
            evidence: "trellara-protocol::barrier_reconstruction_rejects_chunk_checksum_tampering",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "barrier_conflicting_duplicate_messages",
            failure_point:
                "a replay delivers a duplicate chunk, manifest, or commit marker whose contents no longer match the previously buffered transaction boundary",
            invariant: "conflicting_duplicate_barrier_messages_fail_closed",
            boundary_mode: "strict_chunked_or_partitioned_barrier",
            expected_safety_property:
                "the barrier-aware applier rejects conflicting duplicate chunks, manifests, and commit markers before target apply or stream acknowledgement",
            proof_command:
                "cargo test -p trellara-apply-postgres barrier_worker_rejects_conflicting_duplicate_chunks && cargo test -p trellara-apply-postgres barrier_worker_rejects_conflicting_duplicate_manifests && cargo test -p trellara-apply-postgres barrier_worker_rejects_conflicting_duplicate_commit_markers_before_manifest",
            recovery_command: Some("trellara status --config <flow> --view report --format text"),
            evidence:
                "trellara-apply-postgres::barrier_worker_rejects_conflicting_duplicate_chunks; trellara-apply-postgres::barrier_worker_rejects_conflicting_duplicate_manifests; trellara-apply-postgres::barrier_worker_rejects_conflicting_duplicate_commit_markers_before_manifest",
        }),
    ]
}
