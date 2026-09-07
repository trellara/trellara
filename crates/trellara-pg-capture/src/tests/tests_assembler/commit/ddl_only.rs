use super::*;

#[test]
fn assembler_emits_ddl_only_transaction() {
    let mut assembler = TransactionAssembler::default();
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
            LogicalEvent::Begin {
                transaction_id: "tx-ddl".to_string(),
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
                relation,
                statement: "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;"
                    .to_string(),
                schema_fingerprint_before: 12_345,
                schema_fingerprint_after: 67_890,
            }),
        )
        .expect("ddl");

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("commit")
        .expect("ddl envelope");

    assert_eq!(envelope.transaction_id, "tx-ddl");
    assert!(envelope.changes.is_empty());
    assert_eq!(envelope.ddl_events.len(), 1);
    assert_eq!(envelope.ddl_events[0].transaction_id, "tx-ddl");
    assert_eq!(envelope.ddl_events[0].total_order, 1);
    assert!(envelope.ddl_events[0].target_auto_apply);
    assert_eq!(
        envelope.ddl_events[0].release_gate,
        POST_DDL_DML_RELEASE_GATE
    );
    assert_eq!(envelope.schema_versions.len(), 1);
    assert_eq!(envelope.schema_versions[0].version, 67_890);
    envelope.encode_checked().expect("checked DDL envelope");
}

#[test]
fn observed_non_additive_ddl_defaults_to_manual_review_metadata() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");

    assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-ddl-review".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin");
    assembler
        .apply(
            &config,
            LogicalEvent::observed_ddl(ObservedDdlEvent {
                transaction_id: None,
                operation: DdlOperation::ChangePartitionKey,
                relation,
                statement:
                    "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"store_id\" TYPE bigint;"
                        .to_string(),
                schema_fingerprint_before: 12_345,
                schema_fingerprint_after: 67_890,
            }),
        )
        .expect("ddl");

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("commit")
        .expect("ddl envelope");

    assert_eq!(envelope.ddl_events[0].transaction_id, "tx-ddl-review");
    assert_eq!(envelope.ddl_events[0].total_order, 1);
    assert!(!envelope.ddl_events[0].target_auto_apply);
    assert_eq!(
        envelope.ddl_events[0].release_gate,
        POST_DDL_DML_RELEASE_GATE
    );
    envelope
        .encode_checked()
        .expect("manual review DDL envelope remains valid");
}
