use super::*;

#[test]
fn assembler_preserves_mixed_ddl_and_dml_total_order() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");

    assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-mixed".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin");
    assembler
        .apply(
            &config,
            LogicalEvent::observed_ddl(ObservedDdlEvent {
                transaction_id: None,
                operation: DdlOperation::AddColumn,
                relation: relation.clone(),
                statement: "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;"
                    .to_string(),
                schema_fingerprint_before: 12_345,
                schema_fingerprint_after: 67_890,
            }),
        )
        .expect("ddl");
    assembler
        .apply(
            &config,
            LogicalEvent::Change {
                transaction_id: None,
                relation,
                operation: Operation::Insert,
                replica_identity: ReplicaIdentity::Default,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id", 23, "sale-1", true,
                )])),
            },
        )
        .expect("change");

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("commit")
        .expect("mixed envelope");

    assert_eq!(envelope.ddl_events[0].total_order, 1);
    assert!(envelope.ddl_events[0].target_auto_apply);
    assert_eq!(
        envelope.ddl_events[0].release_gate,
        POST_DDL_DML_RELEASE_GATE
    );
    assert_eq!(envelope.changes[0].total_order, 2);
    assert_eq!(
        envelope.changes[0].idempotency_key,
        "source-a:0/16B6C50:tx-mixed:2"
    );
    envelope.encode_checked().expect("checked mixed envelope");
}
