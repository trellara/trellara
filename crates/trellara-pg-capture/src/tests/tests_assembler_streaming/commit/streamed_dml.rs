use super::*;

#[test]
fn assembler_emits_streamed_transaction_only_on_stream_commit() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
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
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start")
        .is_none());
    assert!(assembler
        .apply(
            &config,
            LogicalEvent::Change {
                transaction_id: Some("42".to_string()),
                relation: relation.clone(),
                operation: Operation::Insert,
                replica_identity: ReplicaIdentity::Default,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id", 23, "sale-1", true,
                )])),
            },
        )
        .expect("stream change")
        .is_none());
    assert!(assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop")
        .is_none());
    assert!(assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: false,
            },
        )
        .expect("stream resume")
        .is_none());
    assert!(assembler
        .apply(
            &config,
            LogicalEvent::Change {
                transaction_id: Some("42".to_string()),
                relation,
                operation: Operation::Update,
                replica_identity: ReplicaIdentity::Default,
                before: None,
                after: Some(RowImage::new(vec![ColumnValue::text(
                    "id", 23, "sale-1", true,
                )])),
            },
        )
        .expect("stream change")
        .is_none());
    assert!(assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop")
        .is_none());
    assert!(assembler
        .streamed
        .get("42")
        .expect("streamed transaction")
        .changes
        .is_spilled());

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

    assert_eq!(envelope.transaction_id, "42");
    assert_eq!(envelope.begin_lsn, "");
    assert_eq!(envelope.commit_lsn, "0/16B6C50");
    assert_eq!(envelope.changes.len(), 2);
    assert_eq!(envelope.changes[0].total_order, 1);
    assert_eq!(envelope.changes[1].total_order, 2);
    assert_eq!(
        envelope.changes[1].idempotency_key,
        "source-a:0/16B6C50:42:2"
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
    envelope
        .encode_checked()
        .expect("checked streamed envelope");
}

#[test]
fn assembler_canonicalizes_stream_commit_lsn_before_envelope_emit() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
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
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("streamed change");
    assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop");

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "00000000/016B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("stream commit")
        .expect("streamed envelope");

    assert_eq!(envelope.commit_lsn, "0/16B6C50");
    assert_eq!(
        envelope.changes[0].idempotency_key,
        "source-a:0/16B6C50:42:1"
    );
}
