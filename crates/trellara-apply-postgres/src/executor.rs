use tokio_postgres::types::ToSql;
use tokio_postgres::Transaction;

use crate::{ApplyError, Result, SqlStatement, SqlValueData};

pub(crate) async fn execute_statement(
    transaction: &Transaction<'_>,
    statement: &SqlStatement,
) -> Result<u64> {
    let null_values = vec![None::<String>; statement.values.len()];
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(statement.values.len());
    for (index, value) in statement.values.iter().enumerate() {
        match &value.value {
            SqlValueData::Null => params.push(&null_values[index]),
            SqlValueData::Text(text) => params.push(text),
            SqlValueData::Binary(bytes) => params.push(bytes),
        }
    }
    let affected_rows = transaction.execute(&statement.sql, &params).await?;
    validate_affected_rows(statement, affected_rows)?;
    Ok(affected_rows)
}

pub(crate) fn validate_affected_rows(statement: &SqlStatement, affected_rows: u64) -> Result<()> {
    if statement.requires_row_match && affected_rows == 0 {
        return Err(ApplyError::NoRowsMatched {
            total_order: statement.total_order,
            operation: statement.operation,
        });
    }
    Ok(())
}
