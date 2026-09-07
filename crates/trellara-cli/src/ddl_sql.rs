use crate::{is_safe_postgres_ident, is_safe_postgres_type};
use crate::{DdlPlanChangeKind, DdlPlanDecision};

pub(crate) fn ddl_target_postgres_sql(
    kind: DdlPlanChangeKind,
    object: &str,
    decision: DdlPlanDecision,
) -> Option<String> {
    if !matches!(
        decision,
        DdlPlanDecision::AutoApply | DdlPlanDecision::StageThenApply
    ) {
        return None;
    }
    let spec = ddl_column_type_spec(object)?;
    match kind {
        DdlPlanChangeKind::AddNullableColumn => Some(format!(
            "ALTER TABLE {}.{} ADD COLUMN IF NOT EXISTS {} {};",
            quote_postgres_ident(&spec.schema),
            quote_postgres_ident(&spec.table),
            quote_postgres_ident(&spec.column),
            spec.sql_type
        )),
        DdlPlanChangeKind::WidenType | DdlPlanChangeKind::IncreaseVarchar => Some(format!(
            "ALTER TABLE {}.{} ALTER COLUMN {} TYPE {};",
            quote_postgres_ident(&spec.schema),
            quote_postgres_ident(&spec.table),
            quote_postgres_ident(&spec.column),
            spec.sql_type
        )),
        _ => None,
    }
}

struct DdlColumnTypeSpec {
    schema: String,
    table: String,
    column: String,
    sql_type: String,
}

fn ddl_column_type_spec(object: &str) -> Option<DdlColumnTypeSpec> {
    let (relation_column, sql_type) = object.rsplit_once(':')?;
    let mut parts = relation_column.split('.');
    let schema = parts.next()?.trim();
    let table = parts.next()?.trim();
    let column = parts.next()?.trim();
    if parts.next().is_some()
        || !is_safe_postgres_ident(schema)
        || !is_safe_postgres_ident(table)
        || !is_safe_postgres_ident(column)
        || !is_safe_postgres_type(sql_type)
    {
        return None;
    }

    Some(DdlColumnTypeSpec {
        schema: schema.to_string(),
        table: table.to_string(),
        column: column.to_string(),
        sql_type: sql_type.trim().to_string(),
    })
}

fn quote_postgres_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
