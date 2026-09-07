use bytes::Bytes;
use trellara_protocol::{ColumnValue, RelationId, RowImage};

use crate::error::protocol_byte_label;
use crate::{CaptureError, PgOutputColumn, PgOutputReader, Result};

pub(crate) fn parse_tuple(
    relation: &RelationId,
    columns: &[PgOutputColumn],
    reader: &mut PgOutputReader<'_>,
) -> Result<RowImage> {
    let value_count = usize::from(reader.read_u16()?);
    if value_count != columns.len() {
        return Err(CaptureError::PgOutputParse(format!(
            "tuple column count {} does not match relation {} column count {}",
            value_count,
            relation.display_name(),
            columns.len()
        )));
    }
    let mut values = Vec::with_capacity(value_count);
    for column in columns {
        match reader.read_u8()? {
            b'n' => values.push(ColumnValue::null(
                column.name.clone(),
                column.type_oid,
                column.is_key,
            )),
            b'u' => push_unchanged_toast(relation, column, &mut values)?,
            b't' => {
                let value = reader.read_sized_bytes()?;
                values.push(ColumnValue::text(
                    column.name.clone(),
                    column.type_oid,
                    String::from_utf8_lossy(value).into_owned(),
                    column.is_key,
                ));
            }
            b'b' => {
                let value = reader.read_sized_bytes()?;
                values.push(ColumnValue::binary(
                    column.name.clone(),
                    column.type_oid,
                    Bytes::copy_from_slice(value),
                    column.is_key,
                ));
            }
            tag => {
                return Err(CaptureError::PgOutputParse(format!(
                    "unsupported tuple value tag {}",
                    protocol_byte_label(tag)
                )));
            }
        }
    }
    Ok(RowImage::new(values))
}

fn push_unchanged_toast(
    relation: &RelationId,
    column: &PgOutputColumn,
    values: &mut Vec<ColumnValue>,
) -> Result<()> {
    if column.is_key {
        return Err(CaptureError::PgOutputParse(format!(
            "key column {} in relation {} was omitted as unchanged TOAST; cannot safely identify UPDATE/DELETE rows",
            column.name,
            relation.display_name()
        )));
    }
    values.push(ColumnValue::unchanged_toast(
        column.name.clone(),
        column.type_oid,
        false,
    ));
    Ok(())
}
