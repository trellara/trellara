use super::*;

#[test]
fn pgoutput_decoder_names_invalid_insert_tuple_boundary_by_raw_byte() {
    let mut decoder = decoder_with_sales_relation();
    let mut message = vec![b'I'];
    message.extend_from_slice(&16_384_u32.to_be_bytes());
    message.push(0x01);

    let error = decoder
        .decode(&message)
        .expect_err("invalid insert tuple tag");

    assert!(
        matches!(error, CaptureError::PgOutputParse(message) if message.contains("insert expected new tuple tag N, got 0x01"))
    );
}

#[test]
fn pgoutput_decoder_names_invalid_update_tuple_boundary_by_raw_byte() {
    let mut decoder = decoder_with_sales_relation();
    let mut message = vec![b'U'];
    message.extend_from_slice(&16_384_u32.to_be_bytes());
    message.push(0x01);

    let error = decoder
        .decode(&message)
        .expect_err("invalid update tuple tag");

    assert!(
        matches!(error, CaptureError::PgOutputParse(message) if message.contains("update expected tuple tag K, O, or N, got 0x01"))
    );
}

#[test]
fn pgoutput_decoder_names_invalid_update_new_tuple_boundary_by_raw_byte() {
    let mut decoder = decoder_with_sales_relation();
    let mut message = vec![b'U'];
    message.extend_from_slice(&16_384_u32.to_be_bytes());
    message.push(b'K');
    message.extend_from_slice(&1_u16.to_be_bytes());
    message.push(b't');
    message.extend_from_slice(&6_i32.to_be_bytes());
    message.extend_from_slice(b"sale-1");
    message.push(0x01);

    let error = decoder
        .decode(&message)
        .expect_err("invalid update new tuple tag");

    assert!(
        matches!(error, CaptureError::PgOutputParse(message) if message.contains("update expected new tuple tag N, got 0x01"))
    );
}

#[test]
fn pgoutput_decoder_names_invalid_delete_tuple_boundary_by_raw_byte() {
    let mut decoder = decoder_with_sales_relation();
    let mut message = vec![b'D'];
    message.extend_from_slice(&16_384_u32.to_be_bytes());
    message.push(0x01);

    let error = decoder
        .decode(&message)
        .expect_err("invalid delete tuple tag");

    assert!(
        matches!(error, CaptureError::PgOutputParse(message) if message.contains("delete expected tuple tag K or O, got 0x01"))
    );
}
