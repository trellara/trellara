use super::*;

#[test]
fn assembler_rejects_stream_stop_without_active_transaction() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    assert!(matches!(
        assembler.apply(&config, LogicalEvent::StreamStop),
        Err(CaptureError::StreamStopWithoutStart)
    ));
}

#[test]
fn assembler_rejects_new_stream_start_before_stop_without_switching_active_transaction() {
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

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "43".to_string(),
                first_segment: true,
            },
        ),
        Err(CaptureError::StreamStartBeforeStop {
            transaction_id,
            active_transaction_id,
        }) if transaction_id == "43" && active_transaction_id == "42"
    ));

    assembler
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("active transaction still accepts original stream event");
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
        .expect("stream commit")
        .expect("streamed envelope");

    assert_eq!(envelope.transaction_id, "42");
    assert_eq!(envelope.changes.len(), 1);
}

#[test]
fn assembler_rejects_same_stream_start_before_stop_without_discarding_transaction() {
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
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: false,
            },
        ),
        Err(CaptureError::StreamStartBeforeStop {
            transaction_id,
            active_transaction_id,
        }) if transaction_id == "42" && active_transaction_id == "42"
    ));
    assert!(assembler.streamed.contains_key("42"));
}
