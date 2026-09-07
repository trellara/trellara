use super::*;

#[test]
fn partition_planning_fails_without_key_column() {
    let envelope = envelope_with("tx-1", vec![sample_change(1)]);

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::MissingPartitionKey { total_order: 1, .. })
    ));
}

#[test]
fn partition_planning_fails_on_null_key() {
    let mut change = sale_change(1, "store-104");
    change.after = Some(RowImage::new(vec![ColumnValue::null(
        "store_id", 25, false,
    )]));
    let envelope = envelope_with("tx-1", vec![change]);

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::NullPartitionKey { total_order: 1, .. })
    ));
}

#[test]
fn partition_planning_rejects_unsupported_partition_key_value_kind() {
    let mut change = sale_change(1, "store-104");
    change.after = Some(RowImage::new(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue {
            name: "store_id".to_string(),
            type_oid: 25,
            value_kind: 99,
            text_value: "store-104".to_string(),
            binary_value: Vec::new().into(),
            is_key: false,
        },
    ]));
    let envelope = envelope_with("tx-1", vec![change]);

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::UnsupportedValueKind {
            total_order: 1,
            column,
            value_kind: 99,
        }) if column == "store_id"
    ));
}

#[test]
fn partition_planning_rejects_unsupported_primary_key_value_kind_for_null_fallback() {
    let mut change = sale_change(1, "store-104");
    change.after = Some(RowImage::new(vec![
        ColumnValue {
            name: "id".to_string(),
            type_oid: 23,
            value_kind: 99,
            text_value: "sale-1".to_string(),
            binary_value: Vec::new().into(),
            is_key: true,
        },
        ColumnValue::null("store_id", 25, false),
    ]));
    let envelope = envelope_with("tx-1", vec![change]);

    assert!(matches!(
        plan_partitioned_transaction(
            &envelope,
            &partition_config(PartitionNullKeyPolicy::DeriveFromPrimaryKey),
        ),
        Err(ProtocolError::UnsupportedValueKind {
            total_order: 1,
            column,
            value_kind: 99,
        }) if column == "id"
    ));
}
