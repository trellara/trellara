use super::*;

pub(in crate::tests) fn inspect_envelope() -> TransactionEnvelope {
    use trellara_protocol::{
        AffectedTable, ChangeRecord, ManifestBoundaryMode, ManifestPartition, ReplicaIdentity,
        RowImage, StrictEnvelope, TransactionManifest,
    };

    let orders = trellara_protocol::RelationId::new(1, "public", "orders");
    let payments = trellara_protocol::RelationId::new(2, "public", "payments");
    let changes = vec![
        ChangeRecord {
            transaction_id: "tx-inspect".to_string(),
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            relation: Some(orders.clone()),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Full as i32,
            before: None,
            after: Some(RowImage::new(Vec::new())),
            idempotency_key: "source-a:0/16B6C50:tx-inspect:1".to_string(),
        },
        ChangeRecord {
            transaction_id: "tx-inspect".to_string(),
            total_order: 2,
            table_order: 1,
            partition_order: 1,
            relation: Some(payments.clone()),
            operation: Operation::Update as i32,
            replica_identity: ReplicaIdentity::Full as i32,
            before: Some(RowImage::new(Vec::new())),
            after: Some(RowImage::new(Vec::new())),
            idempotency_key: "source-a:0/16B6C50:tx-inspect:2".to_string(),
        },
        ChangeRecord {
            transaction_id: "tx-inspect".to_string(),
            total_order: 3,
            table_order: 2,
            partition_order: 2,
            relation: Some(orders.clone()),
            operation: Operation::Delete as i32,
            replica_identity: ReplicaIdentity::Full as i32,
            before: Some(RowImage::new(Vec::new())),
            after: None,
            idempotency_key: "source-a:0/16B6C50:tx-inspect:3".to_string(),
        },
    ];
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-inspect".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes,
    });
    envelope.manifest = Some(TransactionManifest {
        transaction_id: "tx-inspect".to_string(),
        source_commit_lsn: "0/16B6C50".to_string(),
        source_commit_timestamp_ms: 1_786_420_000_000,
        global_event_count: 3,
        partitions: vec![
            ManifestPartition {
                id: 0,
                event_count: 1,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 10,
            },
            ManifestPartition {
                id: 2,
                event_count: 2,
                first_total_order: 2,
                last_total_order: 3,
                checksum: 20,
            },
        ],
        affected_tables: vec![
            AffectedTable {
                relation: Some(orders),
                event_count: 2,
            },
            AffectedTable {
                relation: Some(payments),
                event_count: 1,
            },
        ],
        boundary_mode: ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();
    envelope
}
