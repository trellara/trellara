use super::*;

pub(crate) fn relation() -> RelationId {
    RelationId::new(42, "public", TABLE_NAME)
}

pub(crate) fn row(id: &str, amount: &str) -> RowImage {
    RowImage::new(vec![
        ColumnValue::text("id", 25, id, true),
        ColumnValue::text("amount_cents", 25, amount, false),
    ])
}

pub(crate) fn binary_column(name: &str, bytes: &[u8]) -> ColumnValue {
    ColumnValue::binary(name, 17, bytes.to_vec(), false)
}
