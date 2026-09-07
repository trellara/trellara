use super::*;

#[test]
fn row_hash_is_independent_of_column_order() {
    let left = RowSnapshot::from_columns(
        "sale-1",
        &[("id", Some("sale-1")), ("amount_cents", Some("1299"))],
    );
    let right = RowSnapshot::from_columns(
        "sale-1",
        &[("amount_cents", Some("1299")), ("id", Some("sale-1"))],
    );

    assert_eq!(left.row_hash, right.row_hash);
}

#[test]
fn json_row_hash_is_independent_of_column_order() {
    let left = RowSnapshot::from_json_value(
        "sale-1",
        &serde_json::json!({
            "id": "sale-1",
            "amount_cents": "1299",
            "metadata": { "register": "7", "lane": "2" }
        }),
    )
    .expect("json row");
    let right = RowSnapshot::from_json_value(
        "sale-1",
        &serde_json::json!({
            "metadata": { "lane": "2", "register": "7" },
            "amount_cents": "1299",
            "id": "sale-1"
        }),
    )
    .expect("json row");

    assert_eq!(left.row_hash, right.row_hash);
}
