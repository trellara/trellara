use super::*;
use trellara_protocol::{DdlOperation, POST_DDL_DML_RELEASE_GATE};

#[test]
fn assembler_drops_aborted_stream_subtransaction_ddl_events() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");
    record_sales_relation_metadata(&mut assembler, &config);

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
        .apply(
            &config,
            LogicalEvent::Ddl {
                transaction_id: Some("43".to_string()),
                operation: DdlOperation::AddColumn,
                relation: relation.clone(),
                statement: "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"rolled_back\" text;"
                    .to_string(),
                schema_fingerprint_before: 12_345,
                schema_fingerprint_after: 67_890,
                target_auto_apply: true,
                release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
            },
        )
        .expect("subtransaction ddl");
    assembler
        .apply(&config, streamed_insert_event("42", "committed"))
        .expect("parent change");
    assembler
        .apply(
            &config,
            LogicalEvent::StreamAbort {
                transaction_id: "42".to_string(),
                subtransaction_id: "43".to_string(),
            },
        )
        .expect("subtransaction abort");
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

    assert!(envelope.ddl_events.is_empty());
    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(envelope.changes[0].total_order, 1);
    envelope
        .encode_checked()
        .expect("checked streamed envelope");
}
