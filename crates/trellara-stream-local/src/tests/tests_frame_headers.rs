use std::path::Path;

use crate::frame_headers::{
    decode_utf8_field, read_optional_headers, read_required_headers, MAX_RECORD_HEADERS,
};
use crate::frame_io::write_bytes_to_vec;

use super::*;

#[test]
fn required_headers_reject_missing_value_as_corrupt_frame() {
    let path = Path::new("stream.log");
    let mut bytes = Vec::new();
    write_bytes_to_vec(&mut bytes, "header_key", b"trace-id").expect("header key");
    let mut reader = &bytes[..];

    assert!(matches!(
        read_required_headers(&mut reader, path, 1),
        Err(LocalStreamError::CorruptFrame { .. })
    ));
}

#[test]
fn optional_headers_treat_missing_value_as_incomplete_frame() {
    let path = Path::new("stream.log");
    let mut bytes = Vec::new();
    write_bytes_to_vec(&mut bytes, "header_key", b"trace-id").expect("header key");
    let mut reader = &bytes[..];

    assert_eq!(
        read_optional_headers(&mut reader, path, 1).expect("optional headers"),
        None
    );
}

#[test]
fn header_fields_reject_invalid_utf8() {
    let path = Path::new("stream.log");

    assert!(matches!(
        decode_utf8_field(path, vec![0xff]),
        Err(LocalStreamError::Io { .. })
    ));
}

#[test]
fn headers_reject_absurd_count_before_allocation() {
    let path = Path::new("stream.log");
    let mut reader = &[][..];

    assert!(matches!(
        read_optional_headers(&mut reader, path, MAX_RECORD_HEADERS + 1),
        Err(LocalStreamError::CorruptFrame { .. })
    ));
}

#[test]
fn headers_accept_maximum_configured_count() {
    let path = Path::new("stream.log");
    let mut bytes = Vec::new();
    for index in 0..MAX_RECORD_HEADERS {
        write_bytes_to_vec(&mut bytes, "header_key", format!("key-{index}").as_bytes())
            .expect("header key");
        write_bytes_to_vec(
            &mut bytes,
            "header_value",
            format!("value-{index}").as_bytes(),
        )
        .expect("header value");
    }
    let mut reader = &bytes[..];

    let headers = read_required_headers(&mut reader, path, MAX_RECORD_HEADERS).expect("headers");

    assert_eq!(headers.len(), MAX_RECORD_HEADERS as usize);
    assert_eq!(headers[0].key, "key-0");
    assert_eq!(headers[0].value, "value-0");
}
