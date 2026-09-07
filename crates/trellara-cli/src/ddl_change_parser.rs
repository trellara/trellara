use crate::{CliError, DdlPlanChangeKind, Result};

pub(crate) fn parse_ddl_plan_change(input: &str) -> Result<(DdlPlanChangeKind, String)> {
    let Some((kind, object)) = input.split_once(':') else {
        return Err(CliError::InvalidConfig(format!(
            "DDL change '{input}' must use kind:object syntax"
        )));
    };
    let object = object.trim();
    if object.is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "DDL change '{input}' must include an object"
        )));
    }
    let kind = match kind.trim().replace('-', "_").as_str() {
        "add_nullable_column" => DdlPlanChangeKind::AddNullableColumn,
        "add_table" => DdlPlanChangeKind::AddTable,
        "widen_type" => DdlPlanChangeKind::WidenType,
        "increase_varchar" => DdlPlanChangeKind::IncreaseVarchar,
        "drop_column" => DdlPlanChangeKind::DropColumn,
        "rename_column" => DdlPlanChangeKind::RenameColumn,
        "rename_table" => DdlPlanChangeKind::RenameTable,
        "narrow_type" => DdlPlanChangeKind::NarrowType,
        "change_primary_key" => DdlPlanChangeKind::ChangePrimaryKey,
        "change_partition_key" => DdlPlanChangeKind::ChangePartitionKey,
        "add_not_null_column" => DdlPlanChangeKind::AddNotNullColumn,
        _ => DdlPlanChangeKind::Unknown,
    };
    Ok((kind, object.to_string()))
}

pub(crate) fn ddl_change_relation(kind: DdlPlanChangeKind, object: &str) -> Result<String> {
    let relation = match kind {
        DdlPlanChangeKind::AddTable | DdlPlanChangeKind::RenameTable => relation_object(object),
        _ => column_object_relation(object),
    }?;
    Ok(relation)
}

fn relation_object(object: &str) -> Result<String> {
    let parts = object_parts(object);
    if parts.len() == 2 {
        return Ok(format!("{}.{}", parts[0], parts[1]));
    }
    Err(invalid_object(
        object,
        "relation-level DDL must use schema.table",
    ))
}

fn column_object_relation(object: &str) -> Result<String> {
    let parts = object_parts(object);
    if parts.len() >= 3 {
        return Ok(format!("{}.{}", parts[0], parts[1]));
    }
    Err(invalid_object(
        object,
        "column-level DDL must use schema.table.column",
    ))
}

fn object_parts(object: &str) -> Vec<&str> {
    object
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

fn invalid_object(object: &str, reason: &str) -> CliError {
    CliError::InvalidConfig(format!("DDL change object '{object}' is invalid: {reason}"))
}
