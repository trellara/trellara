use std::collections::BTreeMap;

use trellara_protocol::{
    idempotency_key, ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity, RowImage,
    StrictEnvelope, TransactionEnvelope,
};

use super::*;

#[test]
fn file_change_accumulates_file_identity_and_lsn_range() {
    let config = LakePlanConfig::new(vec![crate::LakeTableConfig::new("public", "sales", "id")]);
    let mut files = BTreeMap::new();
    let first = envelope("store-001", "tx-1", "0/16B6C50", 1);
    let second = envelope("store-001", "tx-2", "0/16B6D00", 2);

    let relation = accumulate_file_change(
        &mut files,
        &config,
        "retail",
        &first,
        "store-001:tx-1:0/16B6C50",
        7,
        &first.changes[0],
    )
    .expect("first change");
    accumulate_file_change(
        &mut files,
        &config,
        "retail",
        &second,
        "store-001:tx-2:0/16B6D00",
        7,
        &second.changes[0],
    )
    .expect("second change");

    let file = files.values().next().expect("file accumulator");
    assert_eq!(relation, "public.sales");
    assert_eq!(file.table_name, "retail__public__sales__raw_cdc");
    assert_eq!(file.source_bucket, 7);
    assert_eq!(
        file.source_ids.iter().collect::<Vec<_>>(),
        vec!["store-001"]
    );
    assert_eq!(file.transactions.len(), 2);
    assert_eq!(file.change_count, 2);
    assert_eq!(file.min_commit_lsn.as_deref(), Some("0/16B6C50"));
    assert_eq!(file.max_commit_lsn.as_deref(), Some("0/16B6D00"));
    assert_eq!(file.idempotency_keys.len(), 2);
}

#[test]
fn file_change_counts_duplicate_transaction_checksum_once() {
    let config = LakePlanConfig::new(vec![crate::LakeTableConfig::new("public", "sales", "id")]);
    let mut files = BTreeMap::new();
    let envelope = envelope("store-001", "tx-1", "0/16B6C50", 1);

    for _ in 0..2 {
        accumulate_file_change(
            &mut files,
            &config,
            "retail",
            &envelope,
            "store-001:tx-1:0/16B6C50",
            3,
            &envelope.changes[0],
        )
        .expect("file change");
    }

    let file = files.values().next().expect("file accumulator");
    assert_eq!(file.transactions.len(), 1);
    assert_eq!(file.change_count, 2);
    assert_eq!(file.checksum_rollup, envelope.checksum);
}

fn envelope(
    source_id: &str,
    transaction_id: &str,
    commit_lsn: &str,
    total_order: u32,
) -> TransactionEnvelope {
    let change = ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(RelationId::new(16_384, "public", "sales")),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(RowImage::new(vec![ColumnValue::text(
            "id", 25, "sale-1", true,
        )])),
        idempotency_key: idempotency_key(source_id, commit_lsn, transaction_id, total_order),
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
