use trellara_protocol::{
    ChangeRecord, ColumnValue, DdlEvent, Operation, RelationId, ReplicaIdentity, RowImage,
    StrictEnvelope, TransactionEnvelope,
};

pub(in crate::tests) fn envelope(transaction_id: &str, commit_lsn: &str) -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![ChangeRecord {
            transaction_id: transaction_id.to_string(),
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            relation: Some(RelationId::new(42, "public", "sales")),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            after: Some(RowImage::new(vec![ColumnValue::text(
                "store_id", 25, "store-1", true,
            )])),
            idempotency_key: format!("source:{commit_lsn}:{transaction_id}:1"),
        }],
    })
}

pub(in crate::tests) fn multi_change_envelope(
    transaction_id: &str,
    commit_lsn: &str,
) -> TransactionEnvelope {
    let mut envelope = envelope(transaction_id, commit_lsn);
    envelope.changes = (1..=5)
        .map(|order| ChangeRecord {
            transaction_id: transaction_id.to_string(),
            total_order: order,
            table_order: order,
            partition_order: order,
            relation: Some(RelationId::new(42, "public", "sales")),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            after: Some(RowImage::new(vec![ColumnValue::text(
                "store_id",
                25,
                format!("store-{order}"),
                true,
            )])),
            idempotency_key: format!("source:{commit_lsn}:{transaction_id}:{order}"),
        })
        .collect();
    envelope.finalize_checksum();
    envelope
}

pub(in crate::tests) fn ddl_envelope(
    transaction_id: &str,
    commit_lsn: &str,
) -> TransactionEnvelope {
    let mut envelope = envelope(transaction_id, commit_lsn);
    envelope.changes[0].total_order = 2;
    envelope.changes[0].table_order = 2;
    envelope.changes[0].partition_order = 2;
    envelope.changes[0].idempotency_key = format!("source:{commit_lsn}:{transaction_id}:2");
    envelope.ddl_events = vec![DdlEvent::additive_column(
        transaction_id,
        1,
        RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    envelope
}
