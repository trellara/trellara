use super::*;

#[test]
fn duplicate_transaction_event_order_fails_closed_before_lake_planning() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    let mut duplicate = envelope.changes[0].clone();
    duplicate.idempotency_key = idempotency_key("source-a", "0/16B6C50", "tx-1", 99);
    envelope.changes.push(duplicate);
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("duplicate event order");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(
            trellara_protocol::ProtocolError::DuplicateTransactionEventOrder { total_order: 1 }
        )
    ));
}

#[test]
fn missing_primary_key_fails_current_and_history_planning() {
    let envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(RowImage::new(vec![ColumnValue::text(
            "amount", 25, "10", false,
        )])),
    )]);

    let error = plan_commit(&envelope, &config()).expect_err("missing primary key");

    assert!(matches!(
        error,
        LakeError::MissingPrimaryKey {
            total_order: 1,
            primary_key
        } if primary_key == "id"
    ));
}

#[test]
fn unsupported_change_operation_fails_closed_before_lake_planning() {
    let mut invalid_change = change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    );
    invalid_change.operation = 99;
    let envelope = envelope(vec![invalid_change]);

    let error = plan_commit(&envelope, &config()).expect_err("unsupported operation");

    assert!(matches!(
        error,
        LakeError::UnsupportedOperation {
            total_order: 1,
            operation: 99,
        }
    ));
}

#[test]
fn unspecified_change_operation_fails_closed_before_lake_planning() {
    let mut invalid_change = change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    );
    invalid_change.operation = Operation::Unspecified as i32;
    let envelope = envelope(vec![invalid_change]);

    let error = plan_commit(&envelope, &config()).expect_err("unspecified operation");

    assert!(matches!(
        error,
        LakeError::UnsupportedOperation {
            total_order: 1,
            operation: 0,
        }
    ));
}
