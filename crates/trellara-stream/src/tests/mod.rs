use super::*;
use trellara_protocol::{
    ChangeRecord, ColumnValue, DdlEvent, Operation, RelationId, ReplicaIdentity, RowImage,
    StrictEnvelope, TransactionEnvelope,
};

mod barrier_messages;
mod keys;
mod strict_messages;
mod topics;

pub(super) fn sample_envelope() -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source_a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![ChangeRecord {
            transaction_id: "tx-1".to_string(),
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
            idempotency_key: "source_a:0/16B6C50:tx-1:1".to_string(),
        }],
    })
}
