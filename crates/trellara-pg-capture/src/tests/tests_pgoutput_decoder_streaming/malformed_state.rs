use super::*;

#[test]
fn pgoutput_decoder_rejects_malformed_stream_start_without_opening_state() {
    let mut decoder = PgOutputDecoder::default();
    let mut malformed_start = pgoutput_stream_start_message(42, true);
    malformed_start.push(0);

    assert!(matches!(
        decoder.decode(&malformed_start),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("pgoutput message has 1 trailing bytes")
    ));

    assert_eq!(
        decoder
            .decode(&pgoutput_stream_start_message(42, true))
            .expect("valid stream start after malformed message"),
        Some(LogicalEvent::StreamStart {
            transaction_id: "42".to_string(),
            first_segment: true,
        })
    );
}

#[test]
fn pgoutput_decoder_rejects_invalid_stream_start_flag_without_opening_state() {
    let mut decoder = PgOutputDecoder::default();
    let mut malformed_start = pgoutput_stream_start_message(42, true);
    let flag = malformed_start.last_mut().expect("first segment flag");
    *flag = 2;

    assert!(matches!(
        decoder.decode(&malformed_start),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream start first-segment flag must be 0 or 1")
                && message.contains("0x02")
    ));

    assert_eq!(
        decoder
            .decode(&pgoutput_stream_start_message(42, true))
            .expect("valid stream start after malformed flag"),
        Some(LogicalEvent::StreamStart {
            transaction_id: "42".to_string(),
            first_segment: true,
        })
    );
}

#[test]
fn pgoutput_decoder_rejects_malformed_stream_stop_without_clearing_active_state() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    let mut malformed_stop = pgoutput_stream_stop_message();
    malformed_stop.push(0);

    assert!(matches!(
        decoder.decode(&malformed_stop),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("pgoutput message has 1 trailing bytes")
    ));
    assert!(matches!(
        decoder.decode(&pgoutput_stream_start_message(43, true)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream start for xid 43")
                && message.contains("stream xid 42 is still active")
    ));
}

#[test]
fn pgoutput_decoder_rejects_malformed_stream_commit_without_closing_transaction() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    decoder
        .decode(&pgoutput_stream_stop_message())
        .expect("stream stop");
    let mut malformed_commit = pgoutput_stream_commit_message(42, 0x16B6C00, 0x16B6C50, 123_000);
    malformed_commit.push(0);

    assert!(matches!(
        decoder.decode(&malformed_commit),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("pgoutput message has 1 trailing bytes")
    ));
    assert_eq!(
        decoder
            .decode(&pgoutput_stream_commit_message(
                42, 0x16B6C00, 0x16B6C50, 123_000
            ))
            .expect("valid stream commit after malformed message"),
        Some(LogicalEvent::StreamCommit {
            transaction_id: "42".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            commit_timestamp_ms: 946_684_800_123,
        })
    );
}

#[test]
fn pgoutput_decoder_rejects_malformed_stream_abort_without_clearing_active_state() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_stream_start_message(42, true))
        .expect("stream start");
    let mut malformed_abort = pgoutput_stream_abort_message(42, 42);
    malformed_abort.push(0);

    assert!(matches!(
        decoder.decode(&malformed_abort),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("pgoutput message has 1 trailing bytes")
    ));
    assert!(matches!(
        decoder.decode(&pgoutput_stream_start_message(43, true)),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream start for xid 43")
                && message.contains("stream xid 42 is still active")
    ));
}
