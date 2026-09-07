use trellara_protocol::{
    idempotency_key, ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity, RowImage,
    StrictEnvelope, TransactionEnvelope,
};

pub(super) fn envelope(
    source_id: &str,
    transaction_id: &str,
    commit_lsn: &str,
    sale_id: &str,
    amount_cents: &str,
) -> TransactionEnvelope {
    let change = ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order: 1,
        table_order: 1,
        partition_order: 1,
        relation: Some(RelationId::new(16_384, "public", "sales")),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(RowImage::new(vec![
            ColumnValue::text("id", 25, sale_id, true),
            ColumnValue::text("amount_cents", 20, amount_cents, false),
        ])),
        idempotency_key: idempotency_key(source_id, commit_lsn, transaction_id, 1),
    };
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: source_id.to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "retail".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6C00".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![change],
    })
}

pub(super) fn two_change_envelope(
    source_id: &str,
    transaction_id: &str,
    commit_lsn: &str,
) -> TransactionEnvelope {
    let mut envelope = envelope(source_id, transaction_id, commit_lsn, "sale-2", "99");
    let mut second = envelope.changes[0].clone();
    second.total_order = 2;
    second.table_order = 2;
    second.partition_order = 2;
    second.idempotency_key = idempotency_key(source_id, commit_lsn, transaction_id, 2);
    envelope.changes.push(second);
    envelope.finalize_checksum();
    envelope
}
