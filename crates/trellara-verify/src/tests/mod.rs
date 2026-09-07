use super::*;

mod comparison;
mod row_hashing;
mod sql_generation;
mod watermarks;

pub(super) fn relation() -> RelationId {
    RelationId::new(42, "public", "sales")
}

pub(super) fn row(id: &str, amount: &str) -> RowSnapshot {
    RowSnapshot::from_columns(id, &[("id", Some(id)), ("amount_cents", Some(amount))])
}
