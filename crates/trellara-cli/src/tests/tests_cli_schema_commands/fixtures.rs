use super::*;

pub(super) fn ddl_envelope() -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(trellara_protocol::StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-ddl".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    });
    envelope.ddl_events = vec![trellara_protocol::DdlEvent::additive_column(
        "tx-ddl",
        1,
        trellara_protocol::RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.schema_versions = vec![trellara_protocol::RelationSchemaVersion {
        relation: Some(trellara_protocol::RelationId::new(42, "public", "sales")),
        version: 67_890,
    }];
    envelope.finalize_checksum();
    envelope
}

pub(super) fn mixed_ddl_dml_envelope() -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(trellara_protocol::StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-mixed-ddl".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![mixed_dml_change()],
    });
    envelope.ddl_events = vec![trellara_protocol::DdlEvent::additive_column(
        "tx-mixed-ddl",
        1,
        trellara_protocol::RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    envelope
}

pub(super) fn manual_review_ddl_envelope() -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(trellara_protocol::StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-ddl".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    });
    envelope.ddl_events = vec![trellara_protocol::DdlEvent::manual_review(
        "tx-ddl",
        1,
        trellara_protocol::DdlOperation::ChangePartitionKey,
        trellara_protocol::RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"store_id\" TYPE bigint;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    envelope
}

fn mixed_dml_change() -> trellara_protocol::ChangeRecord {
    trellara_protocol::ChangeRecord {
        transaction_id: "tx-mixed-ddl".to_string(),
        total_order: 2,
        table_order: 1,
        partition_order: 1,
        relation: Some(trellara_protocol::RelationId::new(42, "public", "sales")),
        operation: trellara_protocol::Operation::Insert as i32,
        replica_identity: trellara_protocol::ReplicaIdentity::Default as i32,
        before: None,
        after: Some(trellara_protocol::RowImage::new(vec![
            trellara_protocol::ColumnValue::text("id", 23, "sale-1", true),
            trellara_protocol::ColumnValue::text("discount_code", 25, "LAUNCH", false),
        ])),
        idempotency_key: "source-a:0/16B6C50:tx-mixed-ddl:2".to_string(),
    }
}
