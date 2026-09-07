use super::*;

pub(super) fn assert_chaos_report_pgoutput_contract(summary: &ChaosRunSummary) {
    let relay = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "relay_after_broker_publish_before_source_feedback")
        .expect("relay scenario");
    assert_eq!(relay.invariant, "source_checkpoint_after_broker_ack");
    assert_eq!(relay.boundary_mode, "strict_transaction_order");
    assert!(relay.proof_command.contains("trellara-relay"));
    let schema_change = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "pgoutput_schema_change_during_stream")
        .expect("pgoutput schema change scenario");
    assert_eq!(
        schema_change.invariant,
        "source_schema_fingerprint_fail_closed"
    );
    assert_eq!(schema_change.boundary_mode, "pgoutput_relation_metadata");
    assert!(schema_change
        .proof_command
        .contains("pgoutput_decoder_fails_closed_on_relation_schema_change"));
    assert!(schema_change
        .proof_command
        .contains("pgoutput_decoder_fails_closed_on_schema_change_during_stream"));
    assert!(schema_change
        .evidence
        .contains("pgoutput_decoder_fails_closed_on_schema_change_during_stream"));
    let missing_relation = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "pgoutput_row_without_relation_metadata")
        .expect("pgoutput missing relation metadata scenario");
    assert_eq!(
        missing_relation.invariant,
        "relation_metadata_required_before_rows"
    );
    assert_eq!(missing_relation.boundary_mode, "pgoutput_relation_metadata");
    assert!(missing_relation
        .proof_command
        .contains("pgoutput_decoder_fails_closed_without_relation_metadata"));
    let dml_mapping = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "pgoutput_dml_truncate_envelope_mapping")
        .expect("pgoutput DML mapping scenario");
    assert_eq!(
        dml_mapping.invariant,
        "pgoutput_dml_maps_to_transaction_envelope"
    );
    assert_eq!(dml_mapping.boundary_mode, "pgoutput_relation_metadata");
    assert!(dml_mapping
        .proof_command
        .contains("pgoutput_decoder_tracks_relation_and_insert_tuple"));
    assert!(dml_mapping
        .proof_command
        .contains("pgoutput_decoder_parses_update_key_and_omits_unchanged_toast"));
    assert!(dml_mapping
        .proof_command
        .contains("pgoutput_decoder_parses_delete_truncate_and_commit_boundaries"));
    assert!(dml_mapping
        .proof_command
        .contains("assembler_expands_truncate_relations_in_source_order"));
    let schema_handoff = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "source_schema_handoff_recovery")
        .expect("source schema handoff scenario");
    assert_eq!(
        schema_handoff.invariant,
        "schema_drift_requires_fresh_snapshot_handoff"
    );
    assert_eq!(schema_handoff.boundary_mode, "pgoutput_relation_metadata");
    assert!(schema_handoff
        .proof_command
        .contains("source_schema_drift_recovery_action_requires_fresh_handoff"));
    assert!(schema_handoff
        .proof_command
        .contains("contract_test_scripts_schema_handoff_when_pinned_fingerprint_drifts"));
    let streamed_transaction = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "pgoutput_streamed_transaction_spills_until_commit")
        .expect("pgoutput streamed transaction scenario");
    assert_eq!(
        streamed_transaction.invariant,
        "streamed_transaction_visible_only_after_stream_commit"
    );
    assert_eq!(
        streamed_transaction.boundary_mode,
        "pgoutput_protocol_v2_streaming"
    );
    assert!(streamed_transaction
        .proof_command
        .contains("assembler_emits_streamed_transaction_only_on_stream_commit"));
    let stream_abort = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "pgoutput_stream_abort_discards_partial_changes")
        .expect("pgoutput stream abort scenario");
    assert_eq!(
        stream_abort.invariant,
        "stream_abort_discards_partial_changes"
    );
    assert_eq!(stream_abort.boundary_mode, "pgoutput_protocol_v2_streaming");
    assert!(stream_abort
        .proof_command
        .contains("assembler_drops_aborted_stream_subtransaction_changes"));
    assert!(stream_abort
        .proof_command
        .contains("assembler_removes_spill_file_after_stream_abort"));
    let keepalive_ack = summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == "pgoutput_keepalive_reports_last_durable_ack")
        .expect("pgoutput keepalive ack scenario");
    assert_eq!(
        keepalive_ack.invariant,
        "keepalive_ack_uses_last_durable_boundary"
    );
    assert_eq!(keepalive_ack.boundary_mode, "pgoutput_replication_protocol");
    assert!(keepalive_ack
        .proof_command
        .contains("standby_status_update_payload_reports_acknowledged_lsn_for_every_watermark"));
}
