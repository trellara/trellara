use crate::{ChangeRecord, ProtocolError, RowImage, ValueKind};

pub(crate) fn partition_key_bytes(
    change: &ChangeRecord,
    key_column: &str,
) -> Result<Vec<u8>, ProtocolError> {
    let row = change.after.as_ref().or(change.before.as_ref()).ok_or(
        ProtocolError::MissingPartitionRowImage {
            total_order: change.total_order,
        },
    )?;

    partition_key_bytes_from_row(row, change.total_order, key_column)
}

pub(crate) fn optional_partition_key_bytes_from_row(
    row: &RowImage,
    total_order: u32,
    key_column: &str,
) -> Result<Option<Vec<u8>>, ProtocolError> {
    match partition_key_bytes_from_row(row, total_order, key_column) {
        Ok(key) => Ok(Some(key)),
        Err(ProtocolError::NullPartitionKey { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn partition_key_bytes_from_row(
    row: &RowImage,
    total_order: u32,
    key_column: &str,
) -> Result<Vec<u8>, ProtocolError> {
    let column = row
        .columns
        .iter()
        .find(|column| column.name == key_column)
        .ok_or_else(|| ProtocolError::MissingPartitionKey {
            total_order,
            column: key_column.to_string(),
        })?;

    match value_kind(column, total_order)? {
        ValueKind::Text => Ok(column.text_value.as_bytes().to_vec()),
        ValueKind::Binary => Ok(column.binary_value.to_vec()),
        ValueKind::Null => Err(ProtocolError::NullPartitionKey {
            total_order,
            column: key_column.to_string(),
        }),
        ValueKind::Unspecified | ValueKind::UnchangedToast => {
            Err(ProtocolError::MissingPartitionKey {
                total_order,
                column: key_column.to_string(),
            })
        }
    }
}

pub(crate) fn primary_key_bytes_from_row(
    row: &RowImage,
    total_order: u32,
) -> Result<Vec<u8>, ProtocolError> {
    let mut key_columns = row
        .columns
        .iter()
        .filter(|column| column.is_key)
        .collect::<Vec<_>>();
    key_columns.sort_by(|left, right| left.name.cmp(&right.name));

    if key_columns.is_empty() {
        return Err(ProtocolError::MissingPartitionKey {
            total_order,
            column: "primary key".to_string(),
        });
    }

    let mut bytes = Vec::new();
    for column in key_columns {
        bytes.extend_from_slice(column.name.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&column.type_oid.to_be_bytes());
        bytes.push(column.value_kind as u8);
        match value_kind(column, total_order)? {
            ValueKind::Text => bytes.extend_from_slice(column.text_value.as_bytes()),
            ValueKind::Binary => bytes.extend_from_slice(&column.binary_value),
            ValueKind::Null | ValueKind::Unspecified | ValueKind::UnchangedToast => {
                return Err(ProtocolError::MissingPartitionKey {
                    total_order,
                    column: column.name.clone(),
                });
            }
        }
        bytes.push(0xff);
    }

    Ok(bytes)
}

fn value_kind(column: &crate::ColumnValue, total_order: u32) -> Result<ValueKind, ProtocolError> {
    ValueKind::try_from(column.value_kind).map_err(|_| ProtocolError::UnsupportedValueKind {
        total_order,
        column: column.name.clone(),
        value_kind: column.value_kind,
    })
}
