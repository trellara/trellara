use super::*;

#[test]
fn assembler_emits_committed_transaction() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");
    let schema_fingerprint = 12_345;

    assert!(assembler
        .apply(
            &config,
            LogicalEvent::RelationMetadata {
                relation: relation.clone(),
                schema_fingerprint,
            },
        )
        .expect("relation metadata")
        .is_none());

    assert!(assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-1".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin")
        .is_none());

    assert!(assembler
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
        .expect("change")
        .is_none());

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("commit")
        .expect("committed envelope");

    assert_eq!(envelope.transaction_id, "tx-1");
    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(
        envelope.changes[0].idempotency_key,
        "source-a:0/16B6C50:tx-1:1"
    );
    assert_eq!(envelope.schema_versions.len(), 1);
    assert_eq!(
        envelope.schema_versions[0]
            .relation
            .as_ref()
            .expect("schema version relation"),
        &RelationId::new(16_384, "public", "sales")
    );
    assert_eq!(envelope.schema_versions[0].version, schema_fingerprint);
    envelope.verify_checksum().expect("valid checksum");
}

#[test]
fn assembler_rejects_committed_change_without_relation_metadata() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");

    assert!(assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-missing-schema".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin")
        .is_none());

    assert!(assembler
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
        .expect("change")
        .is_none());

    let error = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect_err("missing relation metadata rejected");

    assert!(matches!(
        error,
        CaptureError::MissingRelationSchemaVersion {
            relation_oid: 16_384
        }
    ));
}
