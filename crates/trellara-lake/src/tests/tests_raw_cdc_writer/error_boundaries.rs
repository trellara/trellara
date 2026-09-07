use super::*;

#[test]
fn raw_cdc_epoch_writer_rejects_conflicting_duplicate_idempotency() {
    let first = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "10", "2026-01-01")),
        )],
    );
    let mut conflicting = envelope_for(
        "store-001",
        "tx-2",
        "0/16B6C70",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "99", "2026-01-01")),
        )],
    );
    conflicting.changes[0].idempotency_key = first.changes[0].idempotency_key.clone();
    conflicting.finalize_checksum();

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[first, conflicting])
        .expect_err("conflicting duplicate");

    assert!(matches!(
        error,
        LakeError::ConflictingDuplicate { idempotency_key }
            if idempotency_key.contains("store-001")
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_zero_source_buckets() {
    let envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(0), &config(), &[envelope])
        .expect_err("invalid buckets");

    assert!(matches!(error, LakeError::InvalidSourceBucketCount));
}

#[test]
fn raw_cdc_epoch_writer_rejects_duplicate_transaction_event_order() {
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

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("duplicate event order");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(
            trellara_protocol::ProtocolError::DuplicateTransactionEventOrder { total_order: 1 }
        )
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_missing_idempotency_key() {
    let missing_key = change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    );
    let mut envelope = envelope(vec![missing_key]);
    envelope.changes[0].idempotency_key = " ".to_string();
    envelope.finalize_checksum();

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("missing idempotency key");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(trellara_protocol::ProtocolError::InvalidChangeField(error))
            if error.total_order == 1
                && error.field == "idempotency_key"
                && error.reason.contains("empty")
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_padded_idempotency_key() {
    let padded_key = change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    );
    let mut envelope = envelope(vec![padded_key]);
    envelope.changes[0].idempotency_key = format!(" {} ", envelope.changes[0].idempotency_key);
    envelope.finalize_checksum();

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("padded idempotency key");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(trellara_protocol::ProtocolError::InvalidChangeField(error))
            if error.total_order == 1
                && error.field == "idempotency_key"
                && error.reason.contains("surrounding whitespace")
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_unsupported_change_operation() {
    let mut invalid_change = change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    );
    invalid_change.operation = 99;
    let envelope = envelope(vec![invalid_change]);

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("unsupported operation");

    assert!(matches!(
        error,
        LakeError::UnsupportedOperation {
            total_order: 1,
            operation: 99,
        }
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_unspecified_change_operation() {
    let mut invalid_change = change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    );
    invalid_change.operation = Operation::Unspecified as i32;
    let envelope = envelope(vec![invalid_change]);

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("unspecified operation");

    assert!(matches!(
        error,
        LakeError::UnsupportedOperation {
            total_order: 1,
            operation: 0,
        }
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_unsupported_after_value_kind() {
    let envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(RowImage::new(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            unsupported_column("amount"),
        ])),
    )]);

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("unsupported after value kind");

    assert!(matches!(
        error,
        LakeError::UnsupportedValueKind { column, value_kind: 99 } if column == "amount"
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_unsupported_before_value_kind() {
    let envelope = envelope(vec![change(
        Operation::Delete,
        1,
        Some(RowImage::new(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            unsupported_column("amount"),
        ])),
        None,
    )]);

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("unsupported before value kind");

    assert!(matches!(
        error,
        LakeError::UnsupportedValueKind { column, value_kind: 99 } if column == "amount"
    ));
}

fn unsupported_column(name: &str) -> ColumnValue {
    ColumnValue {
        name: name.to_string(),
        type_oid: 23,
        value_kind: 99,
        text_value: "10".to_string(),
        binary_value: Vec::new().into(),
        is_key: false,
    }
}
