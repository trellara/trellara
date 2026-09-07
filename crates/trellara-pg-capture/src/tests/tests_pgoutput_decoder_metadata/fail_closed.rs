use super::super::*;

#[test]
fn pgoutput_decoder_fails_closed_on_relation_schema_change() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("amount_cents", 20, false)],
        ))
        .expect("initial relation");

    let error = decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[
                ("id", 25, true),
                ("amount_cents", 20, false),
                ("currency", 25, false),
            ],
        ))
        .expect_err("schema change fails closed");

    assert!(matches!(
        error,
        CaptureError::PgOutputSchemaChanged {
            ref relation,
            previous_fingerprint,
            new_fingerprint,
        } if relation == "public.sales" && previous_fingerprint != new_fingerprint
    ));
    assert!(error.to_string().contains("trellara contract test"));
}

#[test]
fn pgoutput_decoder_fails_closed_on_relation_identity_change() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("amount_cents", 20, false)],
        ))
        .expect("initial relation");

    let error = decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'f',
            &[("id", 25, true), ("amount_cents", 20, false)],
        ))
        .expect_err("identity change fails closed");

    assert!(matches!(
        error,
        CaptureError::PgOutputSchemaChanged {
            ref relation,
            previous_fingerprint,
            new_fingerprint,
        } if relation == "public.sales" && previous_fingerprint != new_fingerprint
    ));
    assert!(error.to_string().contains("fresh schema handoff"));
}

#[test]
fn pgoutput_decoder_fails_closed_on_schema_change_during_stream() {
    let mut decoder = PgOutputDecoder::default();
    assert_eq!(
        decoder
            .decode(&pgoutput_stream_start_message(42, true))
            .expect("stream start"),
        Some(LogicalEvent::StreamStart {
            transaction_id: "42".to_string(),
            first_segment: true,
        })
    );
    decoder
        .decode(&pgoutput_stream_relation_message(
            42,
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("amount_cents", 20, false)],
        ))
        .expect("initial streamed relation");

    let error = decoder
        .decode(&pgoutput_stream_relation_message(
            42,
            16_384,
            "public",
            "sales",
            b'd',
            &[
                ("id", 25, true),
                ("amount_cents", 20, false),
                ("currency", 25, false),
            ],
        ))
        .expect_err("streamed schema change fails closed");

    assert!(matches!(
        error,
        CaptureError::PgOutputSchemaChanged {
            ref relation,
            previous_fingerprint,
            new_fingerprint,
        } if relation == "public.sales" && previous_fingerprint != new_fingerprint
    ));
    assert!(error.to_string().contains("fresh schema handoff"));
}
