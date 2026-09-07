use trellara_protocol::ChangeRecord;

use crate::plan_row::{is_target_owned, key_columns, relation, row_image, supplied_update_columns};
use crate::plan_sql_fragments::{assignments, key_predicate};
use crate::sql::{qualified_table, SqlStatement, SqlValue};
use crate::{ApplyError, ApplyTablePolicy, Result};

pub(crate) fn plan_update(
    change: &ChangeRecord,
    policy: Option<&ApplyTablePolicy>,
) -> Result<Option<SqlStatement>> {
    let relation = relation(change)?;
    let after = row_image(change, "after", change.after.as_ref())?;
    let key_image = change.before.as_ref().unwrap_or(after);
    let mut key_values = key_columns(key_image, change.total_order)?;
    let mutable_columns = supplied_update_columns(after, key_image)?;
    if mutable_columns.is_empty() {
        return Err(ApplyError::NoMutableColumns {
            total_order: change.total_order,
        });
    }
    let mut set_values = mutable_columns
        .into_iter()
        .filter(|column| !is_target_owned(policy, &column.name))
        .map(SqlValue::from_column)
        .collect::<Result<Vec<_>>>()?;
    if set_values.is_empty() {
        return Ok(None);
    }

    let predicate_start = set_values.len() + 1;
    let assignments = assignments(&set_values)?;
    let predicate = key_predicate(&key_values, predicate_start)?;

    set_values.append(&mut key_values);

    Ok(Some(SqlStatement {
        sql: format!(
            "update {} set {} where {}",
            qualified_table(relation),
            assignments,
            predicate
        ),
        values: set_values,
        requires_row_match: true,
        total_order: change.total_order,
        operation: "update",
    }))
}
