use super::super::*;

#[test]
fn pgoutput_decoder_rejects_malformed_relation_without_registering_metadata() {
    let mut decoder = PgOutputDecoder::default();
    let mut malformed_relation = pgoutput_relation_message(
        16_384,
        "public",
        "sales",
        b'd',
        &[("id", 25, true), ("amount_cents", 20, false)],
    );
    malformed_relation.push(0);

    assert!(matches!(
        decoder.decode(&malformed_relation),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("pgoutput message has 1 trailing bytes")
    ));

    assert!(matches!(
        decoder.decode(&pgoutput_insert_message(
            16_384,
            &[TupleValue::text("sale-1"), TupleValue::text("1299")]
        )),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("relation metadata for oid 16384 has not arrived")
    ));
}

#[test]
fn pgoutput_decoder_rejects_unknown_relation_replica_identity() {
    let mut decoder = PgOutputDecoder::default();

    let error = decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'x',
            &[("id", 25, true)],
        ))
        .expect_err("unknown replica identity");

    assert!(matches!(
        error,
        CaptureError::PgOutputParse(message)
            if message.contains("unsupported pgoutput replica identity x (0x78)")
    ));
}

#[test]
fn pgoutput_decoder_rejects_unsupported_relation_column_flags() {
    let mut decoder = PgOutputDecoder::default();

    let error = decoder
        .decode(&relation_message_with_column_flags(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, 0x02)],
        ))
        .expect_err("unsupported column flags");

    assert!(matches!(
        error,
        CaptureError::PgOutputParse(message)
            if message.contains("relation public.sales column id has unsupported flag bits 0x02")
    ));
}

fn relation_message_with_column_flags(
    oid: u32,
    schema: &str,
    table: &str,
    replica_identity: u8,
    columns: &[(&str, u32, u8)],
) -> Vec<u8> {
    let mut message = vec![b'R'];
    message.extend_from_slice(&oid.to_be_bytes());
    put_cstr(&mut message, schema);
    put_cstr(&mut message, table);
    message.push(replica_identity);
    message.extend_from_slice(&(columns.len() as u16).to_be_bytes());
    for (name, type_oid, flags) in columns {
        message.push(*flags);
        put_cstr(&mut message, name);
        message.extend_from_slice(&type_oid.to_be_bytes());
        message.extend_from_slice(&(-1_i32).to_be_bytes());
    }
    message
}

fn put_cstr(message: &mut Vec<u8>, value: &str) {
    message.extend_from_slice(value.as_bytes());
    message.push(0);
}
