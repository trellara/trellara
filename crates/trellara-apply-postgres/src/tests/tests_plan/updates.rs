use super::*;

#[test]
fn plans_update_with_key_predicate() {
    let statement = plan_change(&update_change()).expect("update plan");

    assert_eq!(
            statement.sql,
            "update \"public\".\"sales\" set \"amount_cents\" = $1 where \"id\" is not distinct from $2"
        );
    assert_eq!(
        statement.values,
        vec![
            SqlValue {
                column: "amount_cents".to_string(),
                value: SqlValueData::Text("1499".to_string()),
            },
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-1".to_string()),
            },
        ]
    );
    assert!(statement.requires_row_match);
    assert_eq!(statement.total_order, 2);
    assert_eq!(statement.operation, "update");
}

#[test]
fn update_omits_absent_non_key_columns_for_unchanged_toast() {
    let mut change = update_change();
    change.after = Some(row(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::text("amount_cents", 20, "1499", false),
    ]));

    let statement = plan_change(&change).expect("update plan");

    assert_eq!(
            statement.sql,
            "update \"public\".\"sales\" set \"amount_cents\" = $1 where \"id\" is not distinct from $2"
        );
    assert!(!statement
        .values
        .iter()
        .any(|value| value.column == "receipt_blob"));
    assert!(statement.requires_row_match);
}

#[test]
fn update_omits_explicit_unchanged_toast_marker() {
    let mut change = update_change();
    change.after = Some(row(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::text("amount_cents", 20, "1499", false),
        ColumnValue::unchanged_toast("receipt_blob", 25, false),
    ]));

    let statement = plan_change(&change).expect("update plan");

    assert_eq!(
            statement.sql,
            "update \"public\".\"sales\" set \"amount_cents\" = $1 where \"id\" is not distinct from $2"
        );
    assert!(!statement
        .values
        .iter()
        .any(|value| value.column == "receipt_blob"));
}

#[test]
fn update_without_before_image_uses_after_key_for_default_identity() {
    let mut change = update_change();
    change.before = None;

    let statement = plan_change(&change).expect("update plan");

    assert_eq!(
            statement.sql,
            "update \"public\".\"sales\" set \"amount_cents\" = $1 where \"id\" is not distinct from $2"
        );
    assert_eq!(
        statement.values,
        vec![
            SqlValue {
                column: "amount_cents".to_string(),
                value: SqlValueData::Text("1499".to_string()),
            },
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-1".to_string()),
            },
        ]
    );
}

#[test]
fn key_changing_update_sets_new_key_and_matches_old_key() {
    let mut change = update_change();
    change.before = Some(row(vec![ColumnValue::text("id", 23, "sale-old", true)]));
    change.after = Some(row(vec![
        ColumnValue::text("id", 23, "sale-new", true),
        ColumnValue::text("amount_cents", 20, "1499", false),
    ]));

    let statement = plan_change(&change).expect("update plan");

    assert_eq!(
            statement.sql,
            "update \"public\".\"sales\" set \"id\" = $1, \"amount_cents\" = $2 where \"id\" is not distinct from $3"
        );
    assert_eq!(
        statement.values,
        vec![
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-new".to_string()),
            },
            SqlValue {
                column: "amount_cents".to_string(),
                value: SqlValueData::Text("1499".to_string()),
            },
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-old".to_string()),
            },
        ]
    );
    assert!(statement.requires_row_match);
}

#[test]
fn update_applies_explicit_null_values() {
    let mut change = update_change();
    change.after = Some(row(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::null("receipt_blob", 25, false),
    ]));

    let statement = plan_change(&change).expect("update plan");

    assert_eq!(
            statement.sql,
            "update \"public\".\"sales\" set \"receipt_blob\" = $1 where \"id\" is not distinct from $2"
        );
    assert_eq!(
        statement.values,
        vec![
            SqlValue {
                column: "receipt_blob".to_string(),
                value: SqlValueData::Null,
            },
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-1".to_string()),
            },
        ]
    );
    assert!(statement.requires_row_match);
}
