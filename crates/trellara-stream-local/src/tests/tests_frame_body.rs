use super::*;
use bytes::Bytes;
use trellara_stream::StreamHeader;

#[test]
fn write_record_body_rejects_too_many_headers() {
    let message = StreamMessage {
        topic: "trellara.source.dataset.strict".to_string(),
        key: "tx-too-many-headers".to_string(),
        payload: Bytes::from_static(b"payload"),
        headers: (0..=MAX_RECORD_HEADERS)
            .map(|index| StreamHeader::new(format!("key-{index}"), "value"))
            .collect(),
        partition: Some(0),
        position: None,
    };

    assert!(matches!(
        write_record_body(&message),
        Err(LocalStreamError::FieldTooLarge { field: "headers" })
    ));
}

#[test]
fn write_record_body_accepts_maximum_configured_headers() {
    let message = StreamMessage {
        topic: "trellara.source.dataset.strict".to_string(),
        key: "tx-max-headers".to_string(),
        payload: Bytes::from_static(b"payload"),
        headers: (0..MAX_RECORD_HEADERS)
            .map(|index| StreamHeader::new(format!("key-{index}"), "value"))
            .collect(),
        partition: Some(0),
        position: None,
    };

    let body = write_record_body(&message).expect("record body");

    assert!(!body.is_empty());
}
