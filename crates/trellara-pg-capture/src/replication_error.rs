use fallible_iterator::FallibleIterator;
use postgres_protocol::message::backend::ErrorResponseBody;

use crate::{CaptureError, Result};

pub(crate) fn postgres_error_message(body: &ErrorResponseBody) -> Result<String> {
    let mut fields = body.fields();
    while let Some(field) = fields.next()? {
        if field.type_() == b'M' {
            return Ok(String::from_utf8_lossy(field.value_bytes()).into_owned());
        }
    }
    Ok("unknown postgres error".to_string())
}

pub(crate) fn postgres_error_message_raw(body: &[u8]) -> Result<String> {
    let mut message = "unknown postgres error".to_string();
    let mut remaining = body;
    while let Some((&field_type, rest)) = remaining.split_first() {
        remaining = rest;
        if field_type == 0 {
            break;
        }
        let Some(end) = remaining.iter().position(|byte| *byte == 0) else {
            return Err(CaptureError::ReplicationProtocol(
                "unterminated postgres error field".to_string(),
            ));
        };
        if field_type == b'M' {
            message = String::from_utf8_lossy(&remaining[..end]).into_owned();
        }
        remaining = &remaining[end + 1..];
    }
    Ok(message)
}
