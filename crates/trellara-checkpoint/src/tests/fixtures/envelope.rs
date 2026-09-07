use super::*;
use proptest::prelude::*;
use trellara_protocol::{
    ChangeRecord, Operation, RelationId, ReplicaIdentity, StrictEnvelope, TransactionEnvelope,
};

pub(in crate::tests) fn sample_envelope() -> TransactionEnvelope {
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
            after: None,
            idempotency_key: "source_a:0/16B6C50:tx-1:1".to_string(),
        }],
    })
}

pub(in crate::tests) fn transaction_key_strategy() -> impl Strategy<Value = TransactionKey> {
    (
        "[a-z][a-z0-9]{0,8}",
        "[a-z][a-z0-9]{0,8}",
        "[a-z][a-z0-9]{0,8}",
        "[a-z][a-z0-9-]{0,16}",
        1u64..(1u64 << 40),
    )
        .prop_map(
            |(source_id, database_id, dataset_id, transaction_id, commit_lsn)| TransactionKey {
                source_id,
                database_id,
                dataset_id,
                transaction_id,
                commit_lsn: lsn_string(commit_lsn),
            },
        )
}
