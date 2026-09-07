use super::*;

#[test]
fn chaos_report_html_renders_seed_suite_and_failure_matrix() {
    let summary = ChaosRunSummary::default();
    let html = render_chaos_report_html(&summary);

    assert!(html.contains("Trellara Correctness Report"));
    assert!(html.contains("Deterministic failure proof for Postgres CDC"));
    assert!(html.contains("Report Identity"));
    assert!(html.contains(CORRECTNESS_REPORT_VERSION));
    assert!(html.contains("Performance Envelope"));
    assert!(html.contains("8 minute estimate within 10 minute budget"));
    assert!(html.contains("Stream spill location"));
    assert!(html.contains("./target/trellara-spill"));
    assert!(html.contains("pgoutput protocol v2 streaming"));
    assert!(html.contains("Local indexed replay"));
    assert!(html.contains("Enterprise Proof Review Path"));
    assert!(html.contains("trellara check --config &lt;flow&gt; --format text"));
    assert!(html.contains("trellara inspect-transaction --file &lt;envelope.pb&gt; --format text"));
    assert!(html.contains("manifest boundary_mode names strict chunk or partition semantics"));
    assert!(html.contains("durable-boundary failure harnesses and observability assertions"));
    assert!(html.contains("24-hour large-transaction soak"));
    assert!(html.contains("trellara pilot-package --config &lt;flow&gt;"));
    assert!(html.contains("proof-bundle.md and manifest.json"));
    assert!(html.contains("publish_ack_loss"));
    assert!(html.contains("source_slot_abandoned_idle_timeout"));
    assert!(html.contains("inactive_slot_cleanup_posture_is_explicit"));
    assert!(html.contains("local_stream_index_rebuild"));
    assert!(html.contains("local_index_rebuild_preserves_replay_offsets"));
    assert!(html.contains("missing_index_is_rebuilt_for_offset_replay"));
    assert!(html.contains("stale_index_discovers_durable_tail_without_truncating"));
    assert!(html.contains("inspect_reports_corrupt_index_rebuild"));
    assert!(html.contains("last valid replay offset"));
    assert!(html.contains("inspect_reports_torn_tail_bytes_before_recovery_append"));
    assert!(html.contains("local_stream_inspect_summary_reports_depth_and_pending_messages"));
    assert!(html.contains("pgoutput_schema_change_during_stream"));
    assert!(html.contains("pgoutput_row_without_relation_metadata"));
    assert!(html.contains("relation_metadata_required_before_rows"));
    assert!(html.contains("pgoutput_dml_truncate_envelope_mapping"));
    assert!(html.contains("pgoutput_dml_maps_to_transaction_envelope"));
    assert!(html.contains("pgoutput_stream_abort_discards_partial_changes"));
    assert!(html.contains("stream_abort_discards_partial_changes"));
    assert!(html.contains("pgoutput_keepalive_reports_last_durable_ack"));
    assert!(html.contains("keepalive_ack_uses_last_durable_boundary"));
    assert!(html.contains("source_schema_handoff_recovery"));
    assert!(html.contains("schema_drift_requires_fresh_snapshot_handoff"));
    assert!(html.contains("source_failover_after_publish_before_ack"));
    assert!(html.contains("source_failover_slot_unsynced"));
    assert!(html.contains("failover_slot_sync_before_promotion"));
    assert!(html.contains("source_failover_slot_disabled"));
    assert!(html.contains("failover_slot_enabled_before_promotion"));
    assert!(html.contains("default_replica_identity_primary_key_apply"));
    assert!(html.contains("unchanged_toast_columns_preserved"));
    assert!(html.contains("target_update_delete_zero_rows"));
    assert!(html.contains("primary_key_predicate_apply_without_full"));
    assert!(html.contains("absent_toast_columns_are_unchanged"));
    assert!(html.contains("no_silent_target_divergence"));
    assert!(html.contains("stream_ack_loss_after_apply"));
    assert!(html.contains("Snapshot Handoff Simulation Suite"));
    assert!(html.contains("Strict Chunk Simulation Suite"));
    assert!(html.contains("Fleet Fan-In Lake Simulation Suite"));
    assert!(html.contains("Lane D Qualification Suite"));
    assert!(html.contains("qualification_source_promotion_while_relay_disconnected"));
    assert!(html.contains("source_failover_slot_to_relay_restart"));
    assert!(html.contains("source_ack_lag_visible"));
    assert!(html.contains("qualification_broker_outage_quorum_loss"));
    assert!(html.contains("broker_quorum_unavailable"));
    assert!(html.contains("qualification_target_restart_during_apply"));
    assert!(html.contains("target_checkpoint_not_advanced_early"));
    assert!(html.contains("qualification_object_store_success_catalog_timeout"));
    assert!(html.contains("pending_catalog_commit"));
    assert!(html.contains("qualification_twenty_four_hour_soak_large_transaction_memory_ceiling"));
    assert!(html.contains("large_transaction_memory_ceiling"));
    assert!(html.contains("Source repository"));
    assert!(html.contains("Workflow run"));
    assert!(html.contains("snapshot_handoff_recorded_before_stream_start"));
    assert!(html.contains("snapshot_ddl_during_table_copy"));
    assert!(html.contains("snapshot_contract_before_handoff"));
    assert!(html.contains("snapshot_relay_crash_during_table_copy"));
    assert!(html.contains("strict_chunk_relay_crash_after_chunks_before_manifest"));
    assert!(html.contains("strict_chunk_manifest_arrives_with_missing_chunk"));
    assert!(html.contains("strict_chunk_manifest_arrives_before_chunks"));
    assert!(html.contains("strict_chunk_target_crash_after_staging_before_commit"));
    assert!(html.contains("strict_chunk_stage_not_visible_before_target_commit"));
    assert!(html.contains("strict_chunk_partial_publish_failure"));
    assert!(html.contains("source_checkpoint_waits_for_all_barrier_messages"));
    assert!(html.contains("strict_chunk_checksum_tampering"));
    assert!(html.contains("chunk_checksum_mismatch_fails_closed"));
    assert!(html.contains("barrier_conflicting_duplicate_messages"));
    assert!(html.contains("conflicting_duplicate_barrier_messages_fail_closed"));
    assert!(html.contains("partitioned_scale_mode"));
    assert!(html.contains("fleet_fanin_offline_stores_publish_with_gaps"));
    assert!(html.contains("fleet_fanin_late_store_recovery"));
    assert!(html.contains("fleet_fanin_duplicate_store_replay"));
    assert!(html.contains("fleet_fanin_conflicting_duplicate_quarantine"));
    assert!(html.contains("fleet_fanin_epoch_completeness"));
    assert!(html.contains("complete_with_gaps"));
    assert!(html.contains("quarantined"));
    assert!(html.contains("trellara partition-watermarks --config &lt;flow&gt;"));
    assert!(html.contains("trellara status --config &lt;flow&gt; --view report --format text"));
    assert!(html.contains("trellara status --config &lt;flow&gt; --view dashboard --format text"));
    assert!(html.contains("cargo test --workspace"));
}

#[test]
fn chaos_report_command_writes_static_html() {
    let output = std::env::temp_dir().join(format!(
        "trellara-correctness-report-{}.html",
        std::process::id()
    ));
    let _ = fs::remove_file(&output);

    let summary = write_chaos_report(&ChaosReportArgs {
        output: output.clone(),
        source_revision: "local".to_string(),
        source_repository: "local".to_string(),
        workflow_run_url: "local".to_string(),
    })
    .expect("write report");

    let html = fs::read_to_string(&output).expect("read report");
    assert!(summary.passed);
    assert_eq!(summary.report_version, CORRECTNESS_REPORT_VERSION);
    assert_eq!(summary.source_revision, "local");
    assert_eq!(summary.source_repository, "local");
    assert_eq!(summary.workflow_run_url, "local");
    assert_eq!(summary.simulation_count, 28);
    assert_eq!(summary.scenario_count, 63);
    assert_eq!(summary.enterprise_review_gate_count, 8);
    assert!(summary.enterprise_review_gates.iter().any(|gate| gate.code
        == "partitioned_visibility"
        && gate
            .proof_surface
            .contains("partition-watermarks --config <flow>")));
    assert_eq!(summary.output, output.display().to_string());
    assert!(html.contains("Replayable Simulation Suite"));
    assert!(html.contains("Snapshot Handoff Simulation Suite"));
    assert!(html.contains("Strict Chunk Simulation Suite"));
    assert!(html.contains("Fleet Fan-In Lake Simulation Suite"));
    assert!(html.contains("Lane D Qualification Suite"));
    assert!(html.contains("Curated Failure Matrix"));
    assert!(html.contains("source_failover_after_publish_before_ack"));
    assert!(html.contains("source_schema_handoff_recovery"));
    assert!(html.contains("pgoutput_row_without_relation_metadata"));
    assert!(html.contains("pgoutput_ddl_only_boundary"));
    assert!(html.contains("pgoutput_mixed_ddl_dml_boundary"));
    assert!(html.contains("pgoutput_dml_truncate_envelope_mapping"));
    assert!(html.contains("pgoutput_stream_abort_discards_partial_changes"));
    assert!(html.contains("pgoutput_keepalive_reports_last_durable_ack"));
    assert!(html.contains("source_failover_slot_unsynced"));
    assert!(html.contains("source_failover_slot_disabled"));
    assert!(html.contains("local_stream_index_rebuild"));
    assert!(html.contains("strict_chunk_target_crash_after_staging_before_commit"));
    assert!(html.contains("strict_chunk_partial_publish_failure"));
    assert!(html.contains("strict_chunk_checksum_tampering"));
    assert!(html.contains("partition_commit_marker_ack_ambiguous"));
    assert!(html.contains("partition_commit_marker_manifest_mismatch"));
    assert!(html.contains("fleet_fanin_conflicting_duplicate_quarantine"));
    assert!(html.contains("qualification_broker_outage_quorum_loss"));
    assert!(html.contains("qualification_object_store_success_catalog_timeout"));

    fs::remove_file(output).expect("remove report");
}

#[test]
fn chaos_report_command_stamps_ci_provenance_when_supplied() {
    let output = std::env::temp_dir().join(format!(
        "trellara-correctness-report-ci-{}.html",
        std::process::id()
    ));
    let _ = fs::remove_file(&output);

    let summary = write_chaos_report(&ChaosReportArgs {
        output: output.clone(),
        source_revision: "abc123".to_string(),
        source_repository: "trellara/trellara".to_string(),
        workflow_run_url: "https://github.com/trellara/trellara/actions/runs/42".to_string(),
    })
    .expect("write report");

    let html = fs::read_to_string(&output).expect("read report");
    assert_eq!(summary.source_revision, "abc123");
    assert_eq!(summary.source_repository, "trellara/trellara");
    assert_eq!(
        summary.workflow_run_url,
        "https://github.com/trellara/trellara/actions/runs/42"
    );
    assert!(html.contains("abc123"));
    assert!(html.contains("trellara/trellara"));
    assert!(html.contains("https://github.com/trellara/trellara/actions/runs/42"));

    fs::remove_file(output).expect("remove report");
}
