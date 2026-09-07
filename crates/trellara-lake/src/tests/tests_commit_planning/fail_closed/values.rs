use super::*;

#[test]
fn unsupported_primary_key_value_kind_fails_closed_before_lake_planning() {
    let envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(RowImage::new(vec![
            ColumnValue {
                name: "id".to_string(),
                type_oid: 23,
                value_kind: 99,
                text_value: "sale-1".to_string(),
                binary_value: Vec::new().into(),
                is_key: true,
            },
            ColumnValue::text("amount", 25, "10", false),
        ])),
    )]);

    let error = plan_commit(&envelope, &config()).expect_err("unsupported value kind");

    assert!(matches!(
        error,
        LakeError::UnsupportedValueKind { column, value_kind: 99 } if column == "id"
    ));
}

#[test]
fn unsupported_projected_value_kind_fails_closed_before_lake_planning() {
    let envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(RowImage::new(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            ColumnValue {
                name: "amount".to_string(),
                type_oid: 25,
                value_kind: 99,
                text_value: "10".to_string(),
                binary_value: Vec::new().into(),
                is_key: false,
            },
        ])),
    )]);

    let error = plan_commit(&envelope, &config()).expect_err("unsupported value kind");

    assert!(matches!(
        error,
        LakeError::UnsupportedValueKind { column, value_kind: 99 } if column == "amount"
    ));
}
