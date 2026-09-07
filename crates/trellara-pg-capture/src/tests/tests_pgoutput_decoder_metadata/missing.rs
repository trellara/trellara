use super::super::*;

#[test]
fn pgoutput_decoder_fails_closed_without_relation_metadata() {
    let mut decoder = PgOutputDecoder::default();
    assert!(matches!(
        decoder.decode(&pgoutput_insert_message(
            99,
            &[TupleValue::text("sale-1")]
        )),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("relation metadata for oid 99")
    ));
}
