use super::*;

#[test]
fn assembler_rejects_empty_stream_start_transaction_id_before_opening() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: " ".to_string(),
                first_segment: true,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id.trim().is_empty()
            && reason.contains("stream.transaction_id")
            && reason.contains("empty")
    ));
    assert!(!assembler.streamed.contains_key(" "));
}

#[test]
fn assembler_rejects_padded_stream_start_transaction_id_before_opening() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: " 42 ".to_string(),
                first_segment: true,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == " 42 "
            && reason.contains("stream.transaction_id")
            && reason.contains("surrounding whitespace")
    ));
    assert!(!assembler.streamed.contains_key(" 42 "));
}

#[test]
fn assembler_rejects_empty_stream_commit_transaction_id_without_discarding_open_transaction() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");
    assembler
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("streamed change");
    assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop");

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: " ".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id.trim().is_empty()
            && reason.contains("stream.transaction_id")
            && reason.contains("empty")
    ));
    assert!(assembler.streamed.contains_key("42"));
}

#[test]
fn assembler_rejects_padded_stream_commit_transaction_id_without_discarding_open_transaction() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");
    assembler
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("streamed change");
    assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop");

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: " 42 ".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == " 42 "
            && reason.contains("stream.transaction_id")
            && reason.contains("surrounding whitespace")
    ));
    assert!(assembler.streamed.contains_key("42"));
}

#[test]
fn assembler_rejects_empty_stream_abort_subtransaction_id_without_discarding_parent() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::StreamAbort {
                transaction_id: "42".to_string(),
                subtransaction_id: " ".to_string(),
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id.trim().is_empty()
            && reason.contains("stream.subtransaction_id")
            && reason.contains("empty")
    ));
    assert!(assembler.streamed.contains_key("42"));
}
