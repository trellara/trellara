use super::*;

#[test]
fn assembler_preserves_streamed_ddl_and_dml_total_order_across_segments() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");

    assembler
        .apply(
            &config,
            LogicalEvent::RelationMetadata {
                relation: relation.clone(),
                schema_fingerprint: 67_890,
            },
        )
        .expect("relation metadata");
    assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");
    assembler
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("first streamed change");
    assembler
        .apply(
            &config,
            LogicalEvent::Ddl {
                transaction_id: Some("42".to_string()),
                operation: DdlOperation::AddColumn,
                relation: relation.clone(),
                statement: "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;"
                    .to_string(),
                schema_fingerprint_before: 12_345,
                schema_fingerprint_after: 67_890,
                target_auto_apply: true,
                release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
            },
        )
        .expect("streamed ddl");
    assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop");
    assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: false,
            },
        )
        .expect("stream resume");
    assembler
        .apply(&config, streamed_insert_event("42", "sale-2"))
        .expect("second streamed change");
    assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop");

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("stream commit")
        .expect("streamed envelope");

    assert_eq!(envelope.changes.len(), 2);
    assert_eq!(envelope.ddl_events.len(), 1);
    assert_eq!(envelope.changes[0].total_order, 1);
    assert_eq!(envelope.ddl_events[0].total_order, 2);
    assert_eq!(envelope.changes[1].total_order, 3);
    assert_eq!(envelope.schema_versions.len(), 1);
    assert_eq!(envelope.schema_versions[0].version, 67_890);
    assert_eq!(
        envelope.changes[1].idempotency_key,
        "source-a:0/16B6C50:42:3"
    );
    envelope
        .encode_checked()
        .expect("checked streamed mixed envelope");
}
