use std::collections::BTreeSet;

use crate::{Result, StreamError, StreamMessage};

pub fn validate_message_shape(message: &StreamMessage) -> Result<()> {
    validate_record_key(&message.key)?;
    validate_headers(message)
}

fn validate_record_key(key: &str) -> Result<()> {
    if key.trim().is_empty() {
        return invalid_message_field("key", "must not be empty");
    }
    if key.trim() != key {
        return invalid_message_field("key", "must not contain surrounding whitespace");
    }
    Ok(())
}

fn validate_headers(message: &StreamMessage) -> Result<()> {
    let mut seen = BTreeSet::new();
    for header in &message.headers {
        if header.key.trim().is_empty() {
            return invalid_message_field("header.key", "must not be empty");
        }
        if header.key.trim() != header.key {
            return invalid_message_field("header.key", "must not contain surrounding whitespace");
        }
        if !seen.insert(header.key.as_str()) {
            return invalid_message_field("header.key", "must not appear more than once");
        }
    }
    Ok(())
}

fn invalid_message_field<T>(field: &'static str, reason: impl Into<String>) -> Result<T> {
    Err(StreamError::InvalidMessageField {
        field,
        reason: reason.into(),
    })
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::{validate_message_shape, StreamHeader, StreamMessage};

    fn message() -> StreamMessage {
        StreamMessage {
            topic: "trellara.source.dataset.strict".to_string(),
            key: "tx-1".to_string(),
            payload: Bytes::new(),
            headers: vec![StreamHeader::new("trellara.transaction_id", "tx-1")],
            partition: Some(0),
            position: None,
        }
    }

    #[test]
    fn message_shape_rejects_empty_key() {
        let mut message = message();
        message.key = " ".to_string();

        assert!(validate_message_shape(&message).is_err());
    }

    #[test]
    fn message_shape_rejects_duplicate_header_keys() {
        let mut message = message();
        message
            .headers
            .push(StreamHeader::new("trellara.transaction_id", "tx-1"));

        assert!(validate_message_shape(&message).is_err());
    }
}
