use trellara_protocol::{ChangeRecord, Operation};

use crate::LakeError;

pub(crate) fn validate_change_operation(change: &ChangeRecord) -> Result<Operation, LakeError> {
    let operation =
        Operation::try_from(change.operation).map_err(|_| LakeError::UnsupportedOperation {
            total_order: change.total_order,
            operation: change.operation,
        })?;
    if operation == Operation::Unspecified {
        return Err(LakeError::UnsupportedOperation {
            total_order: change.total_order,
            operation: change.operation,
        });
    }
    Ok(operation)
}
