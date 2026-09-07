use trellara_stream::StreamMessage;

use crate::{LocalStreamError, Result};

pub(crate) fn validate_header_payload(
    message: &StreamMessage,
    offset: i64,
    field: &'static str,
    payload: impl Into<String>,
) -> Result<()> {
    let header = header_value(message, offset, field)?;
    let payload = payload.into();
    if header == payload {
        Ok(())
    } else {
        Err(LocalStreamError::BarrierHeaderPayloadMismatch {
            topic: message.topic.clone(),
            offset,
            field,
            header,
            payload,
        })
    }
}

fn header_value(message: &StreamMessage, offset: i64, field: &'static str) -> Result<String> {
    let key = format!("trellara.{field}");
    let mut matches = message.headers.iter().filter(|header| header.key == key);
    let Some(header) = matches.next() else {
        return Ok(String::new());
    };
    if matches.next().is_some() {
        return Err(LocalStreamError::DuplicateBarrierHeader {
            topic: message.topic.clone(),
            offset,
            field,
        });
    }
    Ok(header.value.clone())
}
