use crate::reader::pgoutput_tuple_value_len;

use super::*;

#[test]
fn pgoutput_tuple_value_len_accepts_non_negative_lengths() {
    assert_eq!(pgoutput_tuple_value_len(0).expect("zero length"), 0);
    assert_eq!(
        pgoutput_tuple_value_len(i32::MAX).expect("max length"),
        i32::MAX as usize
    );
}

#[test]
fn pgoutput_tuple_value_len_rejects_negative_lengths() {
    let error = pgoutput_tuple_value_len(-1).expect_err("negative length");

    assert!(matches!(
        error,
        CaptureError::PgOutputParse(message) if message.contains("negative")
    ));
}

#[test]
fn read_sized_bytes_rejects_negative_length_without_consuming_value_bytes() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(-1_i32).to_be_bytes());
    payload.extend_from_slice(b"value");
    let mut reader = PgOutputReader::new(&payload);

    let error = reader.read_sized_bytes().expect_err("negative length");

    assert!(matches!(
        error,
        CaptureError::PgOutputParse(message) if message.contains("negative")
    ));
    assert_eq!(reader.remaining_len(), b"value".len());
}
