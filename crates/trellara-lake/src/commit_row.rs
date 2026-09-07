use trellara_protocol::{ColumnValue, RowImage, ValueKind};

use crate::{LakeColumnValue, LakeError, LakeTableConfig, LakeValue};

pub(crate) fn required_image<'a>(
    image: Option<&'a RowImage>,
    total_order: u32,
    materialization: &'static str,
) -> Result<&'a RowImage, LakeError> {
    image.ok_or(LakeError::MissingRowImage {
        total_order,
        materialization,
    })
}

pub(crate) fn primary_key_value(
    image: &RowImage,
    table: &LakeTableConfig,
    total_order: u32,
) -> Result<String, LakeError> {
    image
        .columns
        .iter()
        .find(|column| column.name == table.primary_key)
        .map(lake_value_string)
        .transpose()?
        .ok_or_else(|| LakeError::MissingPrimaryKey {
            total_order,
            primary_key: table.primary_key.clone(),
        })
}

pub(crate) fn project_row(
    image: &RowImage,
    table: &LakeTableConfig,
) -> Result<Vec<LakeColumnValue>, LakeError> {
    image
        .columns
        .iter()
        .filter(|column| !table.excluded_columns.contains(&column.name))
        .map(|column| {
            Ok(LakeColumnValue {
                name: column.name.clone(),
                value: lake_value(column)?,
            })
        })
        .collect()
}

pub(crate) fn lake_value_string(column: &ColumnValue) -> Result<String, LakeError> {
    Ok(match value_kind(column)? {
        ValueKind::Null | ValueKind::Unspecified | ValueKind::UnchangedToast => String::new(),
        ValueKind::Text => column.text_value.clone(),
        ValueKind::Binary => format!("<binary:{}>", column.binary_value.len()),
    })
}

fn lake_value(column: &ColumnValue) -> Result<LakeValue, LakeError> {
    Ok(match value_kind(column)? {
        ValueKind::Null | ValueKind::Unspecified => LakeValue::Null,
        ValueKind::Text => LakeValue::Text(column.text_value.clone()),
        ValueKind::Binary => LakeValue::BinaryByteCount(column.binary_value.len()),
        ValueKind::UnchangedToast => LakeValue::UnchangedToast,
    })
}

pub(crate) fn value_kind(column: &ColumnValue) -> Result<ValueKind, LakeError> {
    ValueKind::try_from(column.value_kind).map_err(|_| LakeError::UnsupportedValueKind {
        column: column.name.clone(),
        value_kind: column.value_kind,
    })
}
