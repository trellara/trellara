use trellara_protocol::{ChangeRecord, ColumnValue, RelationId, RowImage, ValueKind};

use crate::sql::SqlValue;
use crate::{ApplyError, ApplyTablePolicy, Result};

pub(crate) fn relation(change: &ChangeRecord) -> Result<&RelationId> {
    change.relation.as_ref().ok_or(ApplyError::MissingRelation {
        total_order: change.total_order,
    })
}

pub(crate) fn row_image<'a>(
    change: &ChangeRecord,
    image: &'static str,
    row: Option<&'a RowImage>,
) -> Result<&'a RowImage> {
    row.ok_or(ApplyError::MissingRowImage {
        total_order: change.total_order,
        image,
    })
}

pub(crate) fn key_columns(row: &RowImage, total_order: u32) -> Result<Vec<SqlValue>> {
    let values = row
        .columns
        .iter()
        .filter(|column| column.is_key)
        .map(|column| {
            validate_key_column(column, total_order)?;
            SqlValue::from_column(column)
        })
        .collect::<Result<Vec<_>>>()?;

    if values.is_empty() {
        Err(ApplyError::MissingKeyColumns { total_order })
    } else {
        Ok(values)
    }
}

fn validate_key_column(column: &ColumnValue, total_order: u32) -> Result<()> {
    let value_kind =
        ValueKind::try_from(column.value_kind).map_err(|_| ApplyError::UnsupportedValueKind {
            column: column.name.clone(),
            value_kind: column.value_kind,
        })?;
    if value_kind == ValueKind::UnchangedToast {
        return Err(ApplyError::UnchangedToastKeyColumn {
            total_order,
            column: column.name.clone(),
        });
    }
    Ok(())
}

pub(crate) fn supplied_update_columns<'a>(
    after: &'a RowImage,
    key_image: &RowImage,
) -> Result<Vec<&'a ColumnValue>> {
    after
        .columns
        .iter()
        .filter_map(|column| match supplied_update_column(column, key_image) {
            Ok(true) => Some(Ok(column)),
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn supplied_update_column(column: &ColumnValue, key_image: &RowImage) -> Result<bool> {
    let value_kind =
        ValueKind::try_from(column.value_kind).map_err(|_| ApplyError::UnsupportedValueKind {
            column: column.name.clone(),
            value_kind: column.value_kind,
        })?;
    Ok((!column.is_key && value_kind != ValueKind::UnchangedToast)
        || key_column_changed(column, key_image))
}

fn key_column_changed(after_column: &ColumnValue, key_image: &RowImage) -> bool {
    if !after_column.is_key {
        return false;
    }
    key_image
        .columns
        .iter()
        .find(|column| column.is_key && column.name == after_column.name)
        .is_some_and(|before_column| !column_values_equal(after_column, before_column))
}

fn column_values_equal(left: &ColumnValue, right: &ColumnValue) -> bool {
    left.value_kind == right.value_kind
        && left.text_value == right.text_value
        && left.binary_value == right.binary_value
}

pub(crate) fn is_target_owned(policy: Option<&ApplyTablePolicy>, column: &str) -> bool {
    policy
        .map(|policy| {
            policy
                .target_owned_columns
                .iter()
                .any(|owned| owned == column)
        })
        .unwrap_or(false)
}
