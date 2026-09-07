use fallible_iterator::FallibleIterator;
use postgres_protocol::message::backend::DataRowBody;

use crate::replication::ReplicationSlotCreation;
use crate::{CaptureError, Result};

pub(crate) fn replication_slot_creation_from_row(
    body: &DataRowBody,
) -> Result<ReplicationSlotCreation> {
    let columns = text_columns(body)?;
    if columns.len() != 4 {
        return Err(CaptureError::ReplicationProtocol(format!(
            "expected 4 CREATE_REPLICATION_SLOT columns, got {}",
            columns.len()
        )));
    }
    Ok(ReplicationSlotCreation {
        slot_name: required_text_column(&columns, 0, "slot_name")?.to_string(),
        consistent_lsn: required_text_column(&columns, 1, "consistent_point")?.to_string(),
        snapshot_name: columns[2].clone(),
        output_plugin: required_text_column(&columns, 3, "output_plugin")?.to_string(),
    })
}

fn text_columns(body: &DataRowBody) -> Result<Vec<Option<String>>> {
    let buffer = body.buffer();
    let mut columns = Vec::new();
    let mut ranges = body.ranges();
    while let Some(range) = ranges.next()? {
        columns.push(range.map(|range| String::from_utf8_lossy(&buffer[range]).into_owned()));
    }
    Ok(columns)
}

fn required_text_column<'a>(
    columns: &'a [Option<String>],
    index: usize,
    name: &str,
) -> Result<&'a str> {
    columns
        .get(index)
        .and_then(Option::as_deref)
        .ok_or_else(|| {
            CaptureError::ReplicationProtocol(format!(
                "CREATE_REPLICATION_SLOT returned null {name}"
            ))
        })
}
