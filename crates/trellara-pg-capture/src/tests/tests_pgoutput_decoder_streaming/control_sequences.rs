use super::*;

#[test]
fn pgoutput_decoder_parses_streamed_transaction_messages() {
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
    assert!(matches!(
        decoder
            .decode(&pgoutput_stream_relation_message(
                42,
                16_384,
                "public",
                "sales",
                b'd',
                &[("id", 25, true), ("amount_cents", 25, false)],
            ))
            .expect("stream relation"),
        Some(LogicalEvent::RelationMetadata {
            relation,
            schema_fingerprint,
        }) if relation == RelationId::new(16_384, "public", "sales") && schema_fingerprint > 0
    ));

    let event = decoder
        .decode(&pgoutput_stream_insert_message(
            42,
            16_384,
            &[TupleValue::text("sale-1"), TupleValue::text("1299")],
        ))
        .expect("stream insert")
        .expect("event");
    match event {
        LogicalEvent::Change {
            transaction_id,
            relation,
            operation,
            ..
        } => {
            assert_eq!(transaction_id, Some("42".to_string()));
            assert_eq!(relation, RelationId::new(16_384, "public", "sales"));
            assert_eq!(operation, Operation::Insert);
        }
        other => panic!("unexpected event {other:?}"),
    }

    assert_eq!(
        decoder
            .decode(&pgoutput_stream_stop_message())
            .expect("stream stop"),
        Some(LogicalEvent::StreamStop)
    );
    assert_eq!(
        decoder
            .decode(&pgoutput_stream_commit_message(
                42, 0x16B6C00, 0x16B6C50, 123_000
            ))
            .expect("stream commit"),
        Some(LogicalEvent::StreamCommit {
            transaction_id: "42".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            commit_timestamp_ms: 946_684_800_123,
        })
    );
}

#[test]
fn pgoutput_decoder_rejects_invalid_stream_control_sequence() {
    let mut decoder = PgOutputDecoder::default();

    assert!(matches!(
        decoder.decode(&pgoutput_stream_stop_message()),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream stop arrived without an active stream")
    ));

    assert_eq!(
        decoder
            .decode(&pgoutput_stream_start_message(42, true))
            .expect("stream start"),
        Some(LogicalEvent::StreamStart {
            transaction_id: "42".to_string(),
            first_segment: true,
        })
    );

    assert!(matches!(
        decoder.decode(&pgoutput_stream_start_message(43, true)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream start for xid 43")
                && message.contains("stream xid 42 is still active")
    ));

    assert!(matches!(
        decoder.decode(&pgoutput_stream_commit_message(
            42, 0x16B6C00, 0x16B6C50, 123_000
        )),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream commit for xid 42 arrived before stream stop")
    ));
}

#[test]
fn pgoutput_decoder_allows_streamed_subtransaction_change_ids() {
    let mut decoder = PgOutputDecoder::default();

    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    decoder
        .decode(&pgoutput_stream_relation_message(
            42,
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("amount_cents", 25, false)],
        ))
        .expect("stream relation");

    let event = decoder
        .decode(&pgoutput_stream_insert_message(
            43,
            16_384,
            &[TupleValue::text("sale-1"), TupleValue::text("1299")],
        ))
        .expect("stream subtransaction insert")
        .expect("event");

    match event {
        LogicalEvent::Change { transaction_id, .. } => {
            assert_eq!(transaction_id, Some("43".to_string()));
        }
        other => panic!("unexpected event {other:?}"),
    }
}
