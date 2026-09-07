use trellara_protocol::ChangeRecord;

use crate::plan_row::{key_columns, relation, row_image};
use crate::plan_sql_fragments::key_predicate;
use crate::sql::{qualified_table, SqlStatement};
use crate::Result;

pub(crate) fn plan_delete(change: &ChangeRecord) -> Result<SqlStatement> {
    let relation = relation(change)?;
    let before = row_image(change, "before", change.before.as_ref())?;
    let values = key_columns(before, change.total_order)?;
    let predicate = key_predicate(&values, 1)?;

    Ok(SqlStatement {
        sql: format!(
            "delete from {} where {}",
            qualified_table(relation),
            predicate
        ),
        values,
        requires_row_match: true,
        total_order: change.total_order,
        operation: "delete",
    })
}
