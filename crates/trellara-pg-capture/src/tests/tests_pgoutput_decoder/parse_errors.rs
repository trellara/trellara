use super::super::*;

#[test]
fn pgoutput_decoder_rejects_truncate_relation_count_without_oid_bytes() {
    let mut decoder = PgOutputDecoder::default();
    let mut message = vec![b'T'];
    message.extend_from_slice(&u32::MAX.to_be_bytes());
    message.push(0);

    let error = decoder
        .decode(&message)
        .expect_err("truncate relation count should fail before allocation");

    assert!(error
        .to_string()
        .contains("truncate relation count 4294967295 exceeds supported maximum"));
}

#[test]
fn pgoutput_decoder_names_unknown_message_type_by_raw_byte() {
    let mut decoder = PgOutputDecoder::default();

    let error = decoder
        .decode(&[0x01])
        .expect_err("unknown message type should fail closed");

    assert!(
        matches!(error, CaptureError::PgOutputParse(message) if message.contains("unsupported pgoutput message type 0x01"))
    );
}

#[test]
fn pgoutput_decoder_names_unknown_tuple_value_tag_by_raw_byte() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true)],
        ))
        .expect("relation");
    let mut message = vec![b'I'];
    message.extend_from_slice(&16_384_u32.to_be_bytes());
    message.push(b'N');
    message.extend_from_slice(&1_u16.to_be_bytes());
    message.push(0x01);

    let error = decoder
        .decode(&message)
        .expect_err("unknown tuple tag should fail closed");

    assert!(
        matches!(error, CaptureError::PgOutputParse(message) if message.contains("unsupported tuple value tag 0x01"))
    );
}
