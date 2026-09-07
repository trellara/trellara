use apache_iceberg::spec::Transform;

use super::schema::iceberg_type;
use super::support::incompatible;
use super::{DesiredTable, SCHEMA_FINGERPRINT_PROPERTY};
use crate::{IcebergRawCdcColumn, Result};

pub(super) struct Compatibility<'a> {
    pub(super) missing_columns: Vec<&'a IcebergRawCdcColumn>,
    pub(super) fingerprint_matches: bool,
}

pub(super) fn inspect_table<'a>(
    table: &apache_iceberg::table::Table,
    desired: &'a DesiredTable<'a>,
) -> Result<Compatibility<'a>> {
    let schema = table.metadata().current_schema();
    let mut missing_columns = Vec::new();
    for column in desired.columns {
        match schema.field_by_id(column.id) {
            Some(actual)
                if actual.name == column.name
                    && actual.required == column.required
                    && actual.field_type.as_ref() == &iceberg_type(column.column_type) => {}
            Some(actual) => {
                return incompatible(
                    desired,
                    format!(
                        "field id {} expected {}:{} required={}, found {}:{} required={}",
                        column.id,
                        column.name,
                        column.column_type.label(),
                        column.required,
                        actual.name,
                        actual.field_type,
                        actual.required
                    ),
                );
            }
            None if schema.field_by_name(&column.name).is_some() => {
                return incompatible(
                    desired,
                    format!("column {} exists with a different field id", column.name),
                );
            }
            None => missing_columns.push(column),
        }
    }
    for actual in schema.as_struct().fields() {
        if desired.columns.iter().all(|column| column.id != actual.id) && actual.required {
            return incompatible(
                desired,
                format!(
                    "table has writer-unknown required column {} with field id {}",
                    actual.name, actual.id
                ),
            );
        }
    }
    let actual_partition = table.metadata().default_partition_spec().fields();
    if actual_partition.len() != desired.partition_fields.len()
        || actual_partition
            .iter()
            .zip(desired.partition_fields)
            .any(|(actual, expected)| {
                actual.source_id != expected.source_column_id
                    || actual.name != expected.source_column
                    || actual.transform != Transform::Identity
                    || expected.transform != "identity"
            })
    {
        return incompatible(
            desired,
            "partition specification differs from the writer contract".to_string(),
        );
    }
    Ok(Compatibility {
        missing_columns,
        fingerprint_matches: table
            .metadata()
            .properties()
            .get(SCHEMA_FINGERPRINT_PROPERTY)
            .is_some_and(|value| value == desired.schema_fingerprint),
    })
}

pub(super) fn require_fully_compatible(
    table: &apache_iceberg::table::Table,
    desired: &DesiredTable<'_>,
) -> Result<()> {
    let compatibility = inspect_table(table, desired)?;
    if compatibility.missing_columns.is_empty() && compatibility.fingerprint_matches {
        Ok(())
    } else {
        incompatible(
            desired,
            "catalog operation completed without the complete desired schema/fingerprint"
                .to_string(),
        )
    }
}
