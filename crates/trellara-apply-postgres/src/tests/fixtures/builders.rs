use super::*;

pub(in crate::tests) fn relation() -> RelationId {
    RelationId::new(42, "public", "sales")
}

pub(in crate::tests) fn schema_version() -> RelationSchemaVersion {
    RelationSchemaVersion {
        relation: Some(relation()),
        version: 12_345,
    }
}

pub(in crate::tests) fn row(columns: Vec<ColumnValue>) -> RowImage {
    RowImage::new(columns)
}

pub(in crate::tests) fn insert_change() -> ChangeRecord {
    ChangeRecord {
        transaction_id: "tx-1".to_string(),
        total_order: 1,
        table_order: 1,
        partition_order: 1,
        relation: Some(relation()),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(row(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            ColumnValue::text("amount_cents", 20, "1299", false),
        ])),
        idempotency_key: idempotency_key("source", "0/16B6C50", "tx-1", 1),
    }
}

pub(in crate::tests) fn update_change() -> ChangeRecord {
    ChangeRecord {
        transaction_id: "tx-1".to_string(),
        total_order: 2,
        table_order: 2,
        partition_order: 2,
        relation: Some(relation()),
        operation: Operation::Update as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: Some(row(vec![ColumnValue::text("id", 23, "sale-1", true)])),
        after: Some(row(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            ColumnValue::text("amount_cents", 20, "1499", false),
        ])),
        idempotency_key: idempotency_key("source", "0/16B6C50", "tx-1", 2),
    }
}

pub(in crate::tests) fn delete_change() -> ChangeRecord {
    ChangeRecord {
        transaction_id: "tx-1".to_string(),
        total_order: 3,
        table_order: 3,
        partition_order: 3,
        relation: Some(relation()),
        operation: Operation::Delete as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: Some(row(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            ColumnValue::text("amount_cents", 20, "1499", false),
        ])),
        after: None,
        idempotency_key: idempotency_key("source", "0/16B6C50", "tx-1", 3),
    }
}

pub(in crate::tests) fn truncate_change() -> ChangeRecord {
    ChangeRecord {
        transaction_id: "tx-1".to_string(),
        total_order: 4,
        table_order: 4,
        partition_order: 4,
        relation: Some(relation()),
        operation: Operation::Truncate as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: None,
        idempotency_key: idempotency_key("source", "0/16B6C50", "tx-1", 4),
    }
}

pub(in crate::tests) fn target_owned_policy() -> ApplyTablePolicy {
    ApplyTablePolicy {
        relation: relation(),
        target_owned_columns: vec!["amount_cents".to_string()],
    }
}

pub(in crate::tests) fn envelope(transaction_id: &str, commit_lsn: &str) -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
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
            relation: Some(relation()),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            after: Some(row(vec![
                ColumnValue::text("id", 23, "sale-1", true),
                ColumnValue::text("store_id", 25, "store-1", false),
                ColumnValue::text("amount_cents", 20, "1299", false),
            ])),
            idempotency_key: idempotency_key("source", commit_lsn, transaction_id, 1),
        }],
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.finalize_checksum();
    envelope
}

pub(in crate::tests) fn partitioned_messages(
    envelope: &TransactionEnvelope,
) -> (StreamMessage, StreamMessage, StreamMessage) {
    let plan = plan_partitioned_transaction(
        envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    let chunk = plan.chunks.first().expect("chunk");
    (
        StreamMessage::partition_chunk(envelope, chunk).expect("chunk message"),
        StreamMessage::transaction_manifest(envelope, &plan.manifest).expect("manifest message"),
        StreamMessage::commit_marker(
            envelope,
            &TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker"),
        )
        .expect("commit marker message"),
    )
}

pub(in crate::tests) fn strict_chunked_messages(
    envelope: &TransactionEnvelope,
) -> (StreamMessage, StreamMessage, StreamMessage) {
    let plan = plan_strict_chunked_transaction(
        envelope,
        &StrictChunkPlanConfig {
            max_changes_per_chunk: 1,
        },
    )
    .expect("strict chunk plan");
    let chunk = plan.chunks.first().expect("chunk");
    (
        StreamMessage::strict_chunk(envelope, chunk).expect("strict chunk message"),
        StreamMessage::transaction_manifest(envelope, &plan.manifest).expect("manifest message"),
        StreamMessage::commit_marker(
            envelope,
            &TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker"),
        )
        .expect("commit marker message"),
    )
}

pub(in crate::tests) fn replace_header(message: &mut StreamMessage, key: &str, value: &str) {
    let header = message
        .headers
        .iter_mut()
        .find(|header| header.key == key)
        .expect("header");
    header.value = value.to_string();
}

pub(in crate::tests) fn remove_header(message: &mut StreamMessage, key: &str) {
    let original_len = message.headers.len();
    message.headers.retain(|header| header.key != key);
    assert_eq!(message.headers.len() + 1, original_len, "header removed");
}
