use super::*;

#[test]
fn plans_insert_by_column_name() {
    let statement = plan_change(&insert_change()).expect("insert plan");

    assert_eq!(
        statement.sql,
        "insert into \"public\".\"sales\" (\"id\", \"amount_cents\") values ($1, $2)"
    );
    assert_eq!(
        statement.values,
        vec![
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-1".to_string()),
            },
            SqlValue {
                column: "amount_cents".to_string(),
                value: SqlValueData::Text("1299".to_string()),
            },
        ]
    );
    assert!(!statement.requires_row_match);
    assert_eq!(statement.total_order, 1);
    assert_eq!(statement.operation, "insert");
}

#[test]
fn plans_delete_with_key_predicate() {
    let statement = plan_change(&delete_change()).expect("delete plan");

    assert_eq!(
        statement.sql,
        "delete from \"public\".\"sales\" where \"id\" is not distinct from $1"
    );
    assert_eq!(
        statement.values,
        vec![SqlValue {
            column: "id".to_string(),
            value: SqlValueData::Text("sale-1".to_string()),
        }]
    );
    assert!(statement.requires_row_match);
    assert_eq!(statement.total_order, 3);
    assert_eq!(statement.operation, "delete");
}

#[test]
fn plans_truncate_without_row_images() {
    let statement = plan_change(&truncate_change()).expect("truncate plan");

    assert_eq!(statement.sql, "truncate table \"public\".\"sales\"");
    assert!(statement.values.is_empty());
    assert!(!statement.requires_row_match);
    assert_eq!(statement.total_order, 4);
    assert_eq!(statement.operation, "truncate");
}
