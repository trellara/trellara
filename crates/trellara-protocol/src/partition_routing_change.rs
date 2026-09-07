use crate::partition_keys::optional_partition_key_bytes_from_row;
use crate::{ChangeRecord, Operation, ProtocolError};

pub(crate) fn partition_key_changed(
    change: &ChangeRecord,
    operation: Operation,
    key_column: &str,
) -> Result<bool, ProtocolError> {
    if operation != Operation::Update {
        return Ok(false);
    }

    let Some(before) = change.before.as_ref() else {
        return Ok(false);
    };
    let Some(after) = change.after.as_ref() else {
        return Ok(false);
    };

    let before_key = optional_partition_key_bytes_from_row(before, change.total_order, key_column)?;
    let after_key = optional_partition_key_bytes_from_row(after, change.total_order, key_column)?;

    Ok(before_key != after_key)
}

pub(crate) fn change_operation(change: &ChangeRecord) -> Result<Operation, ProtocolError> {
    let operation =
        Operation::try_from(change.operation).map_err(|_| ProtocolError::UnsupportedOperation {
            total_order: change.total_order,
            operation: change.operation,
        })?;
    if operation == Operation::Unspecified {
        return Err(ProtocolError::UnsupportedOperation {
            total_order: change.total_order,
            operation: change.operation,
        });
    }
    Ok(operation)
}
