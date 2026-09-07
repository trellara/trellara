use super::*;

pub(crate) fn insert_change(
    transaction_id: &str,
    total_order: u32,
    id: &str,
    amount: &str,
) -> ChangeRecord {
    insert_change_for_relation(transaction_id, total_order, relation(), id, amount)
}

pub(crate) fn insert_change_with_review(
    transaction_id: &str,
    total_order: u32,
    id: &str,
    amount: &str,
    reviewed_by: &str,
) -> ChangeRecord {
    let mut change = insert_change(transaction_id, total_order, id, amount);
    change
        .after
        .as_mut()
        .expect("after image")
        .columns
        .push(ColumnValue::text("reviewed_by", 25, reviewed_by, false));
    change
}

pub(crate) fn insert_change_with_receipt_bytes(
    transaction_id: &str,
    total_order: u32,
    id: &str,
    amount: &str,
    receipt_bytes: &[u8],
) -> ChangeRecord {
    let mut change = insert_change(transaction_id, total_order, id, amount);
    change
        .after
        .as_mut()
        .expect("after image")
        .columns
        .push(binary_column("receipt_bytes", receipt_bytes));
    change
}

pub(crate) fn insert_change_for_relation(
    transaction_id: &str,
    total_order: u32,
    relation: RelationId,
    id: &str,
    amount: &str,
) -> ChangeRecord {
    ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(relation),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(row(id, amount)),
        idempotency_key: idempotency_key(SOURCE_ID, "0/16B6D28", transaction_id, total_order),
    }
}

pub(crate) fn update_change(
    transaction_id: &str,
    total_order: u32,
    id: &str,
    before_amount: &str,
    after_amount: &str,
) -> ChangeRecord {
    ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(relation()),
        operation: Operation::Update as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: Some(row(id, before_amount)),
        after: Some(row(id, after_amount)),
        idempotency_key: idempotency_key(SOURCE_ID, "0/16B6F80", transaction_id, total_order),
    }
}

pub(crate) fn update_change_with_review(
    transaction_id: &str,
    total_order: u32,
    id: &str,
    before_amount: &str,
    after_amount: &str,
    reviewed_by: &str,
) -> ChangeRecord {
    let mut change = update_change(transaction_id, total_order, id, before_amount, after_amount);
    change
        .after
        .as_mut()
        .expect("after image")
        .columns
        .push(ColumnValue::text("reviewed_by", 25, reviewed_by, false));
    change
}

pub(crate) fn key_changing_update(
    transaction_id: &str,
    total_order: u32,
    before_id: &str,
    after_id: &str,
    after_amount: &str,
) -> ChangeRecord {
    let mut change = update_change(
        transaction_id,
        total_order,
        before_id,
        after_amount,
        after_amount,
    );
    change.before = Some(RowImage {
        columns: vec![ColumnValue::text("id", 25, before_id, true)],
    });
    change.after = Some(row(after_id, after_amount));
    change
}

pub(crate) fn update_change_with_receipt_bytes(
    transaction_id: &str,
    total_order: u32,
    id: &str,
    before_amount: &str,
    after_amount: &str,
    receipt_bytes: &[u8],
) -> ChangeRecord {
    let mut change = update_change(transaction_id, total_order, id, before_amount, after_amount);
    change
        .after
        .as_mut()
        .expect("after image")
        .columns
        .push(binary_column("receipt_bytes", receipt_bytes));
    change
}

pub(crate) fn delete_change(
    transaction_id: &str,
    total_order: u32,
    id: &str,
    amount: &str,
) -> ChangeRecord {
    ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(relation()),
        operation: Operation::Delete as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: Some(row(id, amount)),
        after: None,
        idempotency_key: idempotency_key(SOURCE_ID, "0/16B7000", transaction_id, total_order),
    }
}

pub(crate) fn truncate_change(transaction_id: &str, total_order: u32) -> ChangeRecord {
    ChangeRecord {
        transaction_id: transaction_id.to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(relation()),
        operation: Operation::Truncate as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: None,
        idempotency_key: idempotency_key(SOURCE_ID, "0/16B7100", transaction_id, total_order),
    }
}
