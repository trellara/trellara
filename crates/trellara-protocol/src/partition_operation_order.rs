use crate::{Operation, ProtocolError};

pub(crate) fn operation_order(total_order: u32, operation_code: i32) -> Result<u8, ProtocolError> {
    match Operation::try_from(operation_code).map_err(|_| ProtocolError::UnsupportedOperation {
        total_order,
        operation: operation_code,
    })? {
        Operation::Delete => Ok(0),
        Operation::Update => Ok(1),
        Operation::Insert => Ok(2),
        Operation::Truncate => Ok(3),
        Operation::Unspecified => Err(ProtocolError::UnsupportedOperation {
            total_order,
            operation: operation_code,
        }),
    }
}
