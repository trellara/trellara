use super::*;

#[test]
fn mvp_check_marks_local_config_ready_against_definition() {
    let summary = local_mvp_summary();

    assert!(summary.ready);
    assert_eq!(summary.criterion_count, 14);
    assert_eq!(summary.passed_criterion_count, 14);
    assert!(summary.criteria.iter().all(|criterion| criterion.passed));
    let config_path = workspace_path("examples/retail-fleet/local.yml");
    assert_eq!(
        summary.priority_next_commands,
        vec![
            format!(
                "trellara quickstart --config {} --check --format text",
                config_path.display()
            ),
            format!("trellara pilot-package --config {}", config_path.display())
        ]
    );
}

#[test]
fn mvp_check_covers_core_capture_apply_and_snapshot_criteria() {
    let summary = local_mvp_summary();

    let pgoutput = assert_passed_criterion(&summary, "pgoutput_capture_path");
    assert_contains_all(
        &pgoutput.evidence,
        &[
            "pgoutput.protocol_version=2",
            "pgoutput.streaming=true",
            "streamed transaction scenario covered=true",
        ],
    );
    assert_contains_all(
        &pgoutput.proof_command,
        &["trellara check", "trellara-pg-capture", "slot_plugin_guard"],
    );

    let replica_identity = assert_passed_criterion(&summary, "replica_identity_default_supported");
    assert_contains_all(
        &replica_identity.evidence,
        &[
            "primary-key apply covered=true",
            "key-change apply covered=true",
            "unchanged TOAST preservation covered=true",
        ],
    );
    assert_contains_all(
        &replica_identity.proof_command,
        &[
            "plans_update_with_key_predicate",
            "key_changing_update_sets_new_key_and_matches_old_key",
            "update_omits_absent_non_key_columns_for_unchanged_toast",
            "update_omits_explicit_unchanged_toast_marker",
            "pgoutput_decoder_rejects_omitted_unchanged_key_column",
        ],
    );

    let snapshot = assert_passed_criterion(&summary, "snapshot_stream_handoff_crash_safe");
    assert_contains_all(
        &snapshot.evidence,
        &[
            "10/10 snapshot proof scenarios covered",
            "6 snapshot simulations pass",
        ],
    );
    assert_contains_all(
        &snapshot.proof_command,
        &[
            "trellara-sim snapshot",
            "completed_snapshot_summary",
            "snapshot_run_state",
            "snapshot_table_progress_record_requires_watermark_for_terminal_evidence",
            "postgres_reseed_imports_exported_source_snapshot",
        ],
    );
}

#[test]
fn mvp_check_covers_large_transaction_criteria() {
    let summary = local_mvp_summary();
    let large_transactions = assert_passed_criterion(&summary, "large_transactions_bounded");

    assert_contains_all(
        &large_transactions.evidence,
        &[
            "strict_chunking configured=true",
            "8/8 strict chunk proof scenarios covered",
            "5 strict chunk simulations pass",
            "strict chunk inspection audit documented=true",
        ],
    );
    assert_contains_all(
        &large_transactions.proof_command,
        &[
            "assembler_emits_streamed_transaction_only_on_stream_commit",
            "trellara-sim strict_chunk",
            "strict_chunked_partial_publish_failure",
            "strict_chunk_manifest_property_reconstructs_source_order",
        ],
    );
}
