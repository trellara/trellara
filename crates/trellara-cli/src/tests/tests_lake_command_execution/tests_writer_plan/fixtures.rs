use super::*;
use trellara_protocol::{ChangeRecord, ColumnValue, ReplicaIdentity, RowImage, StrictEnvelope};

pub(crate) fn lake_writer_config_file(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "trellara-lake-writer-{prefix}-config-{}-{}.yml",
        std::process::id(),
        unique_test_suffix()
    ))
}

pub(crate) fn lake_writer_envelope_file(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "trellara-lake-writer-{prefix}-envelope-{}-{}.pb",
        std::process::id(),
        unique_test_suffix()
    ))
}

pub(crate) fn lake_writer_envelope(
    source_id: &str,
    transaction_id: &str,
    idempotency_key: &str,
    include_partition_key: bool,
) -> TransactionEnvelope {
    let relation = trellara_protocol::RelationId::new(1, "public", "sales");
    let mut columns = vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::text("amount", 25, "10", false),
    ];
    if include_partition_key {
        columns.insert(1, ColumnValue::text("store_id", 25, "store-001", false));
    }

    TransactionEnvelope::strict(StrictEnvelope {
        source_id: source_id.to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "retail-sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![ChangeRecord {
            transaction_id: transaction_id.to_string(),
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            relation: Some(relation),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Full as i32,
            before: None,
            after: Some(RowImage::new(columns)),
            idempotency_key: idempotency_key.to_string(),
        }],
    })
}
