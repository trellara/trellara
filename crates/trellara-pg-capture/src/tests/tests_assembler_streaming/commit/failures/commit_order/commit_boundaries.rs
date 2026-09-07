use super::*;

#[test]
fn assembler_rejects_zero_stream_commit_lsn_before_emitting_envelope() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    record_sales_relation_metadata(&mut assembler, &config);

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
                transaction_id: "42".to_string(),
                commit_lsn: "0/0".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == "42" && reason.contains("commit_lsn")
    ));

    assert!(assembler.streamed.contains_key("42"));
    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("valid stream commit after invalid boundary")
        .expect("streamed envelope");

    assert_eq!(envelope.transaction_id, "42");
    assert_eq!(envelope.changes.len(), 1);
}

#[test]
fn assembler_rejects_zero_stream_commit_timestamp_without_discarding_transaction() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    record_sales_relation_metadata(&mut assembler, &config);

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
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 0,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == "42" && reason.contains("commit_timestamp_ms")
    ));

    assert!(assembler.streamed.contains_key("42"));
    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("valid stream commit after invalid timestamp")
        .expect("streamed envelope");

    assert_eq!(envelope.transaction_id, "42");
    assert_eq!(envelope.commit_timestamp_ms, 1_786_420_000_000);
}

#[test]
fn assembler_rejects_stream_commit_before_stop_without_discarding_transaction() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    record_sales_relation_metadata(&mut assembler, &config);

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

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        ),
        Err(CaptureError::StreamCommitBeforeStop {
            transaction_id,
            active_transaction_id,
        }) if transaction_id == "42" && active_transaction_id == "42"
    ));

    assert!(assembler.streamed.contains_key("42"));
    assembler
        .apply(&config, LogicalEvent::StreamStop)
        .expect("stream stop");
    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("stream commit after stop")
        .expect("streamed envelope");

    assert_eq!(envelope.changes.len(), 1);
}
