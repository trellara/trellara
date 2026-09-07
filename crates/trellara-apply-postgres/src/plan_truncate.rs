use trellara_protocol::ChangeRecord;

use crate::plan_row::relation;
use crate::sql::{qualified_table, SqlStatement};
use crate::Result;

pub(crate) fn plan_truncate(change: &ChangeRecord) -> Result<SqlStatement> {
    let relation = relation(change)?;

    Ok(SqlStatement {
        sql: format!("truncate table {}", qualified_table(relation)),
        values: Vec::new(),
        requires_row_match: false,
        total_order: change.total_order,
        operation: "truncate",
    })
}
