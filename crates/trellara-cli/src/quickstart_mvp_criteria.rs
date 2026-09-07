use std::path::Path;

use crate::quickstart_artifacts::{
    partitioned_watermark_artifacts_are_current, strict_chunk_audit_artifacts_are_current,
};
use crate::quickstart_mvp_artifact_criteria::build_artifact_readiness_criteria;
use crate::quickstart_partitioned_scale_proofs::partitioned_scale_proof_command;
use crate::{
    chaos_has_scenario, quickstart_capture_spill_message, ChaosRunSummary, MvpProofCounts,
    MvpReadinessCriterion, QuickstartReadinessSummary, SourceCaptureKind, TrellaraConfig,
    MVP_PARTITIONED_SCALE_SCENARIOS, MVP_SCHEMA_CHANGE_SCENARIOS, MVP_SNAPSHOT_SCENARIOS,
    MVP_SOURCE_FAILOVER_SCENARIOS, MVP_STRICT_CHUNK_SCENARIOS,
};

pub(crate) fn build_mvp_readiness_criteria(
    config: &TrellaraConfig,
    config_display: &str,
    repository_root: &Path,
    quickstart: &QuickstartReadinessSummary,
    chaos: &ChaosRunSummary,
    proof_counts: &MvpProofCounts,
) -> Vec<MvpReadinessCriterion> {
    let mut criteria = vec![
        MvpReadinessCriterion::new(
            "no_broker_verified_flow_under_10_minutes",
            quickstart.ready
                && quickstart
                    .estimated_minutes
                    .is_some_and(|minutes| minutes <= quickstart.time_budget_minutes),
            format!(
                "quickstart readiness is {} with estimated={} minutes and budget={} minutes",
                quickstart.ready,
                quickstart
                    .estimated_minutes
                    .map(|minutes| minutes.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                quickstart.time_budget_minutes
            ),
            format!("trellara quickstart --config {config_display} --check --format text"),
        ),
        MvpReadinessCriterion::new(
            "pgoutput_capture_path",
            config.source.capture == SourceCaptureKind::PgOutput
                && config.source.pgoutput.protocol_version >= 2
                && config.source.pgoutput.streaming
                && chaos
                    .scenarios
                    .iter()
                    .any(|scenario| scenario.name == "pgoutput_streamed_transaction_spills_until_commit"),
            format!(
                "source.capture={:?}; pgoutput.protocol_version={}; pgoutput.streaming={}; streamed transaction scenario covered={}",
                config.source.capture,
                config.source.pgoutput.protocol_version,
                config.source.pgoutput.streaming,
                chaos
                    .scenarios
                    .iter()
                    .any(|scenario| scenario.name == "pgoutput_streamed_transaction_spills_until_commit")
            ),
            format!("trellara check --config {config_display} --format text && cargo test -p trellara-pg-capture --lib slot_plugin_guard"),
        ),
        MvpReadinessCriterion::new(
            "replica_identity_default_supported",
            chaos_has_scenario(chaos, "default_replica_identity_primary_key_apply")
                && chaos_has_scenario(chaos, "default_replica_identity_key_change_apply")
                && chaos_has_scenario(chaos, "unchanged_toast_columns_preserved"),
            format!(
                "primary-key apply covered={}; key-change apply covered={}; unchanged TOAST preservation covered={}",
                chaos_has_scenario(chaos, "default_replica_identity_primary_key_apply"),
                chaos_has_scenario(chaos, "default_replica_identity_key_change_apply"),
                chaos_has_scenario(chaos, "unchanged_toast_columns_preserved")
            ),
            "cargo test -p trellara-apply-postgres plans_update_with_key_predicate && cargo test -p trellara-apply-postgres key_changing_update_sets_new_key_and_matches_old_key && cargo test -p trellara-apply-postgres update_omits_absent_non_key_columns_for_unchanged_toast && cargo test -p trellara-apply-postgres update_omits_explicit_unchanged_toast_marker && cargo test -p trellara-pg-capture pgoutput_decoder_rejects_omitted_unchanged_key_column".to_string(),
        ),
        MvpReadinessCriterion::new(
            "snapshot_stream_handoff_crash_safe",
            proof_counts.snapshot == MVP_SNAPSHOT_SCENARIOS.len()
                && chaos.snapshot_simulations.iter().all(|simulation| simulation.passed),
            format!(
                "{}/{} snapshot proof scenarios covered; {} snapshot simulations pass",
                proof_counts.snapshot,
                MVP_SNAPSHOT_SCENARIOS.len(),
                chaos.snapshot_simulations.len()
            ),
            "cargo test -p trellara-sim snapshot && cargo test -p trellara-cli completed_snapshot_summary && cargo test -p trellara-checkpoint snapshot_run_state && cargo test -p trellara-checkpoint snapshot_table_progress_record_requires_watermark_for_terminal_evidence && cargo test -p trellara-verify postgres_reseed_imports_exported_source_snapshot".to_string(),
        ),
        MvpReadinessCriterion::new(
            "large_transactions_bounded",
            config.source.stream_spill_threshold_changes.unwrap_or_default() > 0
                && config.dataset.strict_chunking.is_some()
                && chaos_has_scenario(chaos, "pgoutput_streamed_transaction_spills_until_commit")
                && proof_counts.strict_chunk == MVP_STRICT_CHUNK_SCENARIOS.len()
                && chaos.strict_chunk_simulations.iter().all(|simulation| simulation.passed)
                && strict_chunk_audit_artifacts_are_current(repository_root),
            format!(
                "{}; strict_chunking configured={}; {}/{} strict chunk proof scenarios covered; {} strict chunk simulations pass; strict chunk inspection audit documented={}",
                quickstart_capture_spill_message(config),
                config.dataset.strict_chunking.is_some(),
                proof_counts.strict_chunk,
                MVP_STRICT_CHUNK_SCENARIOS.len(),
                chaos.strict_chunk_simulations.len(),
                strict_chunk_audit_artifacts_are_current(repository_root)
            ),
            "cargo test -p trellara-pg-capture assembler_emits_streamed_transaction_only_on_stream_commit && cargo test -p trellara-sim strict_chunk && cargo test -p trellara-relay strict_chunked_partial_publish_failure_does_not_advance_checkpoint_or_source_ack && cargo test -p trellara-apply-postgres applied_transaction_insert_fails_closed_on_duplicate_key && cargo test -p trellara-protocol strict_chunk_manifest_property_reconstructs_source_order".to_string(),
        ),
        MvpReadinessCriterion::new(
            "partitioned_scale_mode_proven",
            repository_root.join("examples/retail-fleet/partitioned.yml").exists()
                && proof_counts.partitioned_scale == MVP_PARTITIONED_SCALE_SCENARIOS.len()
                && partitioned_watermark_artifacts_are_current(repository_root),
            format!(
                "examples/retail-fleet/partitioned.yml exists={}; {}/{} partitioned scale proof scenarios covered; partitioned scale evidence surface current={}",
                repository_root.join("examples/retail-fleet/partitioned.yml").exists(),
                proof_counts.partitioned_scale,
                MVP_PARTITIONED_SCALE_SCENARIOS.len(),
                partitioned_watermark_artifacts_are_current(repository_root)
            ),
            partitioned_scale_proof_command(repository_root),
        ),
        MvpReadinessCriterion::new(
            "source_failover_readiness_proven",
            proof_counts.source_failover == MVP_SOURCE_FAILOVER_SCENARIOS.len(),
            format!(
                "{}/{} source failover proof scenarios covered; failover slot posture, sync state, and replay after promotion are represented",
                proof_counts.source_failover,
                MVP_SOURCE_FAILOVER_SCENARIOS.len()
            ),
            "cargo test -p trellara-sim source_failover_after_publish_before_ack_recovers_with_duplicate_replay && cargo test -p trellara-relay invalid_envelope_lsn_does_not_advance_checkpoint_or_source_ack --lib && cargo test -p trellara-cli --lib source_safety_warns_when_failover_slot_is_not_synced && cargo test -p trellara-cli --lib source_safety_warns_when_failover_slot_is_disabled && cargo test -p trellara-cli --lib direct_source_safety_warns_when_failover_slot_is_disabled".to_string(),
        ),
        MvpReadinessCriterion::new(
            "schema_change_recovery_scripted",
            proof_counts.schema_change == MVP_SCHEMA_CHANGE_SCENARIOS.len(),
            format!(
                "{}/{} schema-change proof scenarios covered; pgoutput drift fails closed and recovery requires a fresh audited handoff",
                proof_counts.schema_change,
                MVP_SCHEMA_CHANGE_SCENARIOS.len()
            ),
            "cargo test -p trellara-pg-capture pgoutput_decoder_fails_closed_on_relation_schema_change && cargo test -p trellara-pg-capture pgoutput_decoder_fails_closed_on_schema_change_during_stream && cargo test -p trellara-apply-postgres target_ddl_barrier_rejects_schema_version_evidence_mismatch --lib && cargo test -p trellara-cli schema_ddl_envelope_plan_command_renders_runtime_sequence --lib && cargo test -p trellara-cli ddl_barrier_status_renders_release_blocker_codes --lib && cargo test -p trellara-lake ddl_ack --lib && cargo test -p trellara-cli --lib source_schema_drift_recovery_action_requires_fresh_handoff && cargo test -p trellara-cli --lib contract_test_scripts_schema_handoff_when_pinned_fingerprint_drifts && cargo test -p trellara-sim snapshot_ddl_during_table_copy_withholds_handoff_until_contract_refresh".to_string(),
        ),
    ];
    criteria.extend(build_artifact_readiness_criteria(
        config,
        config_display,
        repository_root,
        chaos,
    ));
    criteria
}
