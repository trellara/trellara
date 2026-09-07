use super::*;
use trellara_protocol::{DdlEvent, DdlOperation, Operation, RelationId, ReplicaIdentity};

#[test]
fn next_total_order_is_one_based() {
    assert_eq!(next_total_order("tx-1", 0).expect("order"), 1);
    assert_eq!(next_total_order("tx-1", 41).expect("order"), 42);
}

#[test]
fn next_total_order_fails_closed_before_u32_wraparound() {
    assert!(matches!(
        next_total_order("tx-huge", u32::MAX as usize),
        Err(CaptureError::TransactionOrderOverflow {
            transaction_id,
            max_supported_changes: u32::MAX,
        }) if transaction_id == "tx-huge"
    ));
}

#[test]
fn compact_event_orders_preserves_sparse_source_order() {
    let changes = vec![pending_change(10)];
    let ddl_events = vec![ddl_event(4)];

    assert_eq!(
        compact_event_orders("tx-sparse", &changes, &ddl_events).expect("order map"),
        vec![4, 10]
    );
}

#[test]
fn compact_event_orders_rejects_duplicate_change_and_ddl_order() {
    let changes = vec![pending_change(4)];
    let ddl_events = vec![ddl_event(4)];

    assert!(matches!(
        compact_event_orders("tx-dup", &changes, &ddl_events),
        Err(CaptureError::DuplicateTransactionEventOrder {
            transaction_id,
            total_order: 4,
        }) if transaction_id == "tx-dup"
    ));
}

fn pending_change(total_order: u32) -> PendingChange {
    PendingChange {
        total_order,
        stream_subtransaction_id: None,
        relation: RelationId::new(16_384, "public", "sales"),
        operation: Operation::Insert,
        replica_identity: ReplicaIdentity::Default,
        before: None,
        after: None,
    }
}

fn ddl_event(total_order: u32) -> PendingTransactionDdlEvent {
    PendingTransactionDdlEvent {
        total_order,
        stream_subtransaction_id: None,
        event: DdlEvent::manual_review(
            "tx-1",
            total_order,
            DdlOperation::Other,
            RelationId::new(16_384, "public", "sales"),
            "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"note\" text;",
            1,
            2,
        ),
    }
}
