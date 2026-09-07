use super::*;

#[test]
fn mvp_check_covers_partitioned_scale_and_source_failover_criteria() {
    let summary = local_mvp_summary();

    let partitioned_scale = assert_passed_criterion(&summary, "partitioned_scale_mode_proven");
    assert_contains_all(
        &partitioned_scale.evidence,
        &[
            "examples/retail-fleet/partitioned.yml exists=true",
            "7/7 partitioned scale proof scenarios covered",
            "partitioned scale evidence surface current=true",
        ],
    );
    assert_contains_all(
        &partitioned_scale.proof_command,
        &[
            "trellara partition-watermarks --config",
            "trellara partition-rebalance-plan --config",
            "barrier_reconstruction_rejects_missing_chunks",
            "partition_watermark_rejects_cross_flow_checkpoint_evidence",
            "partition_visibility_ddl_ack",
            "barrier_reconstruction_rejects_exact_duplicate_chunks",
            "barrier_reconstruction_rejects_duplicate_manifest_partitions",
            "commit_marker_rejects_duplicate_manifest_partitions",
            "partition_visibility",
            "partitioned_scale_readiness",
            "strict_message_contains_envelope_headers_and_key",
            "envelope_headers_include_ddl_event_count_and_release_gates",
            "manifest_message_uses_barrier_topic_and_headers",
            "manifest_message_rejects_envelope_boundary_mismatch",
            "chunk_message_rejects_envelope_transaction_mismatch",
            "barrier_worker_rejects_manifest_evidence_header_mismatch",
            "barrier_worker_rejects_manifest_missing_evidence_header",
            "barrier_worker_rejects_chunk_partitioned_scale_readiness_header_mismatch",
            "barrier_worker_requires_partitioned_scale_readiness_headers",
            "barrier_worker_rejects_chunk_partition_routing_header_mismatch",
            "barrier_worker_rejects_unlisted_chunk_buffered_before_manifest",
            "barrier_worker_rejects_unlisted_chunk_after_manifest",
            "barrier_worker_rejects_corrupt_chunk_before_apply_or_ack",
            "reconstruct_local_barrier_transaction_rejects_chunk_header_payload_mismatch",
            "reconstruct_local_barrier_transaction_rejects_corrupt_chunk_manifest_proof",
            "reconstruct_local_barrier_transaction_rejects_duplicate_partition_offsets",
            "partitioned_ambiguous_commit_marker_publish",
            "barrier_worker_rejects_mismatched_commit_marker",
            "pending_stats_reports_invalid_commit_marker_separately",
            "pending_stats_reports_extra_chunks_separately",
            "apply_summary_exposes_invalid_commit_marker_blocker",
            "manifest_envelope_records_partition_checkpoints_atomically",
            "target_quarantine_repair_replay_applies_after_operator_marks_replay_ready",
            "quarantine_replay_ready_refuses_unknown_transaction_boundary",
            "quarantine_replay_contract_preserves_boundary_evidence",
            "partitioned_mode_explains_transaction_barrier",
            "tests_transaction_inspect",
            "tests_correctness_report",
        ],
    );

    let source_failover = assert_passed_criterion(&summary, "source_failover_readiness_proven");
    assert_contains_all(
        &source_failover.evidence,
        &[
            "3/3 source failover proof scenarios covered",
            "failover slot posture",
        ],
    );
    assert_contains_all(
        &source_failover.proof_command,
        &[
            "source_failover_after_publish_before_ack_recovers_with_duplicate_replay",
            "invalid_envelope_lsn_does_not_advance_checkpoint_or_source_ack",
            "source_safety_warns_when_failover_slot_is_not_synced",
            "source_safety_warns_when_failover_slot_is_disabled",
            "direct_source_safety_warns_when_failover_slot_is_disabled",
        ],
    );
}

#[test]
fn mvp_check_covers_schema_ddl_and_protocol_property_criteria() {
    let summary = local_mvp_summary();

    let schema_change = assert_passed_criterion(&summary, "schema_change_recovery_scripted");
    assert_contains_all(
        &schema_change.evidence,
        &[
            "3/3 schema-change proof scenarios covered",
            "fresh audited handoff",
        ],
    );
    assert_contains_all(
        &schema_change.proof_command,
        &[
            "pgoutput_decoder_fails_closed_on_relation_schema_change",
            "pgoutput_decoder_fails_closed_on_schema_change_during_stream",
            "target_ddl_barrier_rejects_schema_version_evidence_mismatch",
            "schema_ddl_envelope_plan_command_renders_runtime_sequence",
            "ddl_barrier_status_renders_release_blocker_codes",
            "cargo test -p trellara-lake ddl_ack",
            "source_schema_drift_recovery_action_requires_fresh_handoff",
            "contract_test_scripts_schema_handoff_when_pinned_fingerprint_drifts",
            "snapshot_ddl_during_table_copy_withholds_handoff_until_contract_refresh",
        ],
    );

    let ddl_propagation = assert_passed_criterion(&summary, "ddl_propagation_contract_packaged");
    assert_contains_all(
        &ddl_propagation.evidence,
        &[
            "schema-barrier propagation",
            "retry-safe additive target SQL",
            "target/lake/Spark sink ACKs",
            "partition visibility pause semantics",
        ],
    );
    assert_contains_all(
        &ddl_propagation.proof_command,
        &[
            "trellara schema ddl-plan --config",
            "trellara schema ddl-apply-plan --config",
            "trellara schema ddl-envelope-plan --config",
            "trellara schema ddl-barrier status --config",
            "target_ddl_transaction_plan_accepts_retry_safe_additive_sql",
            "--apply-mode auto-safe",
            "make quickstart-proof-check",
        ],
    );

    let protocol_properties =
        assert_passed_criterion(&summary, "protocol_property_tests_cover_invariants");
    assert_contains_all(
        &protocol_properties.evidence,
        &[
            "protocol property scenario covered=true",
            "modular envelope/idempotency/LSN/strict chunk/partitioned/missing/duplicate proptests current=true",
        ],
    );
    assert_eq!(
        protocol_properties.proof_command,
        "cargo test -p trellara-protocol property"
    );
}
