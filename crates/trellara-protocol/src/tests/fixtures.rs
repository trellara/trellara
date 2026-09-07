use super::*;

pub(super) fn sample_change(total_order: u32) -> ChangeRecord {
    let relation = RelationId::new(16_384, "public", "sales");
    let key = idempotency_key("source-a", "0/16B6C50", "tx-1", total_order);
    ChangeRecord {
        transaction_id: "tx-1".to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(relation),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(RowImage::new(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            ColumnValue::text("amount_cents", 20, "1299", false),
        ])),
        idempotency_key: key,
    }
}

pub(super) fn sale_change(total_order: u32, store_id: &str) -> ChangeRecord {
    let mut change = sample_change(total_order);
    change.after = Some(RowImage::new(vec![
        ColumnValue::text("id", 23, format!("sale-{total_order}"), true),
        ColumnValue::text("store_id", 25, store_id, false),
        ColumnValue::text("amount_cents", 20, "1299", false),
    ]));
    change
}

pub(super) fn ownership_move_change(
    total_order: u32,
    old_store_id: &str,
    new_store_id: &str,
) -> ChangeRecord {
    let mut change = sale_change(total_order, new_store_id);
    change.operation = Operation::Update as i32;
    change.before = Some(RowImage::new(vec![
        ColumnValue::text("id", 23, format!("sale-{total_order}"), true),
        ColumnValue::text("store_id", 25, old_store_id, false),
        ColumnValue::text("amount_cents", 20, "1299", false),
    ]));
    change
}

pub(super) fn multi_store_envelope() -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![
            sale_change(1, "store-104"),
            sale_change(2, "store-205"),
            sale_change(3, "store-104"),
            sale_change(4, "store-312"),
        ],
    })
}

pub(super) fn generated_envelope(change_count: u32) -> TransactionEnvelope {
    let transaction_id = "tx-generated";
    let changes = (1..=change_count)
        .map(|order| {
            transaction_sale_change(transaction_id, order, &format!("store-{}", order % 8))
        })
        .collect();
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes,
    })
}

pub(super) fn generated_partitioned_envelope(
    change_count: u32,
    partition_count: u32,
) -> TransactionEnvelope {
    let transaction_id = "tx-generated-partitioned";
    let changes = (1..=change_count)
        .map(|order| {
            transaction_sale_change(
                transaction_id,
                order,
                &format!("store-{}", order % partition_count),
            )
        })
        .collect();
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes,
    })
}

pub(super) fn empty_envelope() -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-empty".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    })
}

pub(super) fn transaction_sale_change(
    transaction_id: &str,
    total_order: u32,
    store_id: &str,
) -> ChangeRecord {
    let mut change = sale_change(total_order, store_id);
    change.transaction_id = transaction_id.to_string();
    change.idempotency_key = idempotency_key("source-a", "0/16B6C50", transaction_id, total_order);
    change
}
