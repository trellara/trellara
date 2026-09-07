use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(super) fn pgoutput_transaction_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_dml_truncate_envelope_mapping",
            failure_point: "pgoutput sends committed INSERT, UPDATE, DELETE, and TRUNCATE messages",
            invariant: "pgoutput_dml_maps_to_transaction_envelope",
            boundary_mode: "pgoutput_relation_metadata",
            expected_safety_property:
                "each pgoutput operation maps to the expected Trellara envelope operation with relation identity, row images, source ordering, and checksum evidence",
            proof_command:
                "cargo test -p trellara-pg-capture pgoutput_decoder_tracks_relation_and_insert_tuple && cargo test -p trellara-pg-capture pgoutput_decoder_parses_update_key_and_omits_unchanged_toast && cargo test -p trellara-pg-capture pgoutput_decoder_parses_delete_truncate_and_commit_boundaries && cargo test -p trellara-pg-capture assembler_expands_truncate_relations_in_source_order",
            recovery_command: Some("trellara inspect-transaction --file <envelope.pb>"),
            evidence:
                "trellara-pg-capture::pgoutput_decoder_tracks_relation_and_insert_tuple; trellara-pg-capture::pgoutput_decoder_parses_update_key_and_omits_unchanged_toast; trellara-pg-capture::pgoutput_decoder_parses_delete_truncate_and_commit_boundaries; trellara-pg-capture::assembler_expands_truncate_relations_in_source_order",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_ddl_only_boundary",
            failure_point:
                "pgoutput observes a committed transaction containing schema change metadata and no DML rows",
            invariant: "ddl_only_transaction_preserves_commit_boundary",
            boundary_mode: "pgoutput_transaction_boundary",
            expected_safety_property:
                "capture emits a valid DDL-only transaction envelope with the source transaction id, commit LSN, schema version evidence, and post-DDL DML release gate",
            proof_command: "cargo test -p trellara-pg-capture assembler_emits_ddl_only_transaction",
            recovery_command: Some(
                "trellara schema ddl-envelope-plan --config <flow> --file <envelope.pb>",
            ),
            evidence: "trellara-pg-capture::assembler_emits_ddl_only_transaction",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_mixed_ddl_dml_boundary",
            failure_point:
                "pgoutput observes schema change metadata and row changes in the same committed source transaction",
            invariant: "mixed_ddl_dml_transaction_preserves_total_order",
            boundary_mode: "pgoutput_transaction_boundary",
            expected_safety_property:
                "capture keeps DDL and DML in one transaction envelope, preserves source total order, and forces downstream release through the DDL barrier",
            proof_command:
                "cargo test -p trellara-pg-capture assembler_preserves_mixed_ddl_and_dml_total_order",
            recovery_command: Some("trellara inspect-transaction --file <envelope.pb>"),
            evidence: "trellara-pg-capture::assembler_preserves_mixed_ddl_and_dml_total_order",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_streamed_transaction_spills_until_commit",
            failure_point:
                "Postgres sends a large transaction as streamed pgoutput fragments before commit",
            invariant: "streamed_transaction_visible_only_after_stream_commit",
            boundary_mode: "pgoutput_protocol_v2_streaming",
            expected_safety_property:
                "capture may spill buffered changes to disk, but emits one committed transaction only after STREAM COMMIT",
            proof_command:
                "cargo test -p trellara-pg-capture assembler_emits_streamed_transaction_only_on_stream_commit",
            recovery_command: None,
            evidence:
                "trellara-pg-capture::assembler_emits_streamed_transaction_only_on_stream_commit",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_stream_abort_discards_partial_changes",
            failure_point:
                "Postgres aborts a streamed pgoutput transaction after the relay has buffered or spilled partial changes",
            invariant: "stream_abort_discards_partial_changes",
            boundary_mode: "pgoutput_protocol_v2_streaming",
            expected_safety_property:
                "aborted stream fragments are removed from memory and spill files, and no partial transaction becomes visible",
            proof_command:
                "cargo test -p trellara-pg-capture assembler_drops_aborted_stream_subtransaction_changes && cargo test -p trellara-pg-capture assembler_removes_spill_file_after_stream_abort",
            recovery_command: Some("trellara relay --config <flow>"),
            evidence:
                "trellara-pg-capture::assembler_drops_aborted_stream_subtransaction_changes; trellara-pg-capture::assembler_removes_spill_file_after_stream_abort",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_keepalive_reports_last_durable_ack",
            failure_point:
                "Postgres requests a standby status reply while the relay has decoded ahead of the last durable Trellara boundary",
            invariant: "keepalive_ack_uses_last_durable_boundary",
            boundary_mode: "pgoutput_replication_protocol",
            expected_safety_property:
                "the standby status update repeats only the last durable acknowledged LSN for written, flushed, and applied watermarks",
            proof_command:
                "cargo test -p trellara-pg-capture standby_status_boundary_uses_only_last_durable_acknowledged_lsn && cargo test -p trellara-pg-capture standby_status_update_payload_reports_acknowledged_lsn_for_every_watermark",
            recovery_command: Some("trellara status --config <flow> --view report --format text"),
            evidence:
                "trellara-pg-capture::standby_status_boundary_uses_only_last_durable_acknowledged_lsn; trellara-pg-capture::standby_status_update_payload_reports_acknowledged_lsn_for_every_watermark",
        }),
    ]
}
