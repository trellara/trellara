use trellara_stream::{StreamHeader, StreamMessage};

use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn required_message_header(
    message: &StreamMessage,
    key: &'static str,
) -> ApplyWorkerResult<String> {
    required_header(&message.headers, key)
}

pub(crate) fn optional_header(
    headers: &[StreamHeader],
    key: &'static str,
) -> ApplyWorkerResult<Option<String>> {
    let mut matches = headers.iter().filter(|header| header.key == key);
    let Some(header) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(ApplyWorkerError::InvalidHeaderField {
            key,
            reason: "must not appear more than once".to_string(),
        });
    }
    Ok(Some(header.value.clone()))
}

pub(crate) fn required_header(
    headers: &[StreamHeader],
    key: &'static str,
) -> ApplyWorkerResult<String> {
    optional_header(headers, key)?.ok_or(ApplyWorkerError::MissingHeader(key))
}
