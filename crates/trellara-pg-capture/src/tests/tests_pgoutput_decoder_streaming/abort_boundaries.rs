use super::*;

#[test]
fn pgoutput_decoder_parses_stream_abort_boundary() {
    let mut decoder = PgOutputDecoder::default();

    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    assert_eq!(
        decoder
            .decode(&pgoutput_stream_abort_message(42, 43))
            .expect("stream abort"),
        Some(LogicalEvent::StreamAbort {
            transaction_id: "42".to_string(),
            subtransaction_id: "43".to_string(),
        })
    );
}

#[test]
fn pgoutput_decoder_rejects_stream_abort_without_open_transaction() {
    let mut decoder = PgOutputDecoder::default();

    assert!(matches!(
        decoder.decode(&pgoutput_stream_abort_message(42, 42)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream abort for xid 42")
                && message.contains("without an open streamed transaction")
    ));
}

#[test]
fn pgoutput_decoder_rejects_stream_commit_without_open_transaction() {
    let mut decoder = PgOutputDecoder::default();

    assert!(matches!(
        decoder.decode(&pgoutput_stream_commit_message(
            42, 0x16B6C00, 0x16B6C50, 123_000
        )),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream commit for xid 42")
                && message.contains("without an open streamed transaction")
    ));
}

#[test]
fn pgoutput_decoder_rejects_non_first_segment_without_open_transaction() {
    let mut decoder = PgOutputDecoder::default();

    assert!(matches!(
        decoder.decode(&pgoutput_stream_start_message(42, false)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("non-first stream segment for xid 42")
                && message.contains("before a first segment")
    ));
}

#[test]
fn pgoutput_decoder_rejects_duplicate_first_segment_after_stop() {
    let mut decoder = PgOutputDecoder::default();

    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    decoder
        .decode(&pgoutput_stream_stop_message())
        .expect("stream stop");

    assert!(matches!(
        decoder.decode(&pgoutput_stream_start_message(42, true)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("first stream segment for xid 42")
                && message.contains("already open")
    ));
}

#[test]
fn pgoutput_decoder_parent_stream_abort_clears_active_stream() {
    let mut decoder = PgOutputDecoder::default();

    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    assert_eq!(
        decoder
            .decode(&pgoutput_stream_abort_message(42, 42))
            .expect("parent stream abort"),
        Some(LogicalEvent::StreamAbort {
            transaction_id: "42".to_string(),
            subtransaction_id: "42".to_string(),
        })
    );

    assert_eq!(
        decoder
            .decode(&pgoutput_stream_start_message(43, true))
            .expect("new stream start"),
        Some(LogicalEvent::StreamStart {
            transaction_id: "43".to_string(),
            first_segment: true,
        })
    );
}

#[test]
fn pgoutput_decoder_subtransaction_stream_abort_keeps_parent_active() {
    let mut decoder = PgOutputDecoder::default();

    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    decoder
        .decode(&pgoutput_stream_abort_message(42, 43))
        .expect("subtransaction abort");

    assert!(matches!(
        decoder.decode(&pgoutput_stream_start_message(44, true)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream start for xid 44")
                && message.contains("stream xid 42 is still active")
    ));
}

#[test]
fn pgoutput_decoder_rejects_stream_abort_for_different_active_xid() {
    let mut decoder = PgOutputDecoder::default();

    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");

    assert!(matches!(
        decoder.decode(&pgoutput_stream_abort_message(43, 43)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream abort for xid 43")
                && message.contains("stream xid 42 is still active")
    ));
}
