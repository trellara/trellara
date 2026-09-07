use super::*;
use trellara_protocol::TransactionEnvelope;

fn start_stream(assembler: &mut TransactionAssembler, config: &TransactionAssemblerConfig) {
    assembler
        .apply(
            config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");
}

fn start_two_open_streams_then_resume_first(
    assembler: &mut TransactionAssembler,
    config: &TransactionAssemblerConfig,
) {
    start_stream(assembler, config);
    assembler
        .apply(config, LogicalEvent::StreamStop)
        .expect("stream 42 stop");
    assembler
        .apply(
            config,
            LogicalEvent::StreamStart {
                transaction_id: "43".to_string(),
                first_segment: true,
            },
        )
        .expect("stream 43 start");
    assembler
        .apply(config, LogicalEvent::StreamStop)
        .expect("stream 43 stop");
    assembler
        .apply(
            config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: false,
            },
        )
        .expect("stream 42 resume");
}

fn assert_empty_stream_event_id_rejected(result: Result<Option<TransactionEnvelope>>) {
    assert!(matches!(
        result,
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id.trim().is_empty()
            && reason.contains("stream.event_transaction_id")
            && reason.contains("empty")
    ));
}

fn assert_other_open_stream_event_id_rejected(result: Result<Option<TransactionEnvelope>>) {
    assert!(matches!(
        result,
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == "43"
            && reason.contains("stream.event_transaction_id")
            && reason.contains("different open streamed transaction")
            && reason.contains("active streamed transaction is 42")
    ));
}

#[test]
fn assembler_rejects_empty_stream_change_transaction_id_without_buffering_change() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    record_sales_relation_metadata(&mut assembler, &config);

    start_stream(&mut assembler, &config);

    assert_empty_stream_event_id_rejected(
        assembler.apply(&config, streamed_insert_event(" ", "malformed")),
    );

    assembler
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("valid streamed change after rejection");
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
        .expect("non-empty envelope");

    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(
        envelope.changes[0].after.as_ref().unwrap().columns[0].text_value,
        "sale-1"
    );
}

#[test]
fn assembler_rejects_other_open_stream_change_transaction_id_without_buffering_change() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();
    record_sales_relation_metadata(&mut assembler, &config);

    start_two_open_streams_then_resume_first(&mut assembler, &config);

    assert_other_open_stream_event_id_rejected(
        assembler.apply(&config, streamed_insert_event("43", "wrong-tx")),
    );

    assembler
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("valid streamed change after rejection");
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
        .expect("non-empty envelope");

    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(
        envelope.changes[0].after.as_ref().unwrap().columns[0].text_value,
        "sale-1"
    );
}

#[test]
fn assembler_rejects_empty_stream_truncate_transaction_id() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    start_stream(&mut assembler, &config);

    assert_empty_stream_event_id_rejected(assembler.apply(
        &config,
        LogicalEvent::Truncate {
            transaction_id: Some(" ".to_string()),
            relations: vec![RelationId::new(16_384, "public", "sales")],
        },
    ));
    assert!(assembler.streamed.contains_key("42"));
}

#[test]
fn assembler_rejects_other_open_stream_truncate_transaction_id() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    start_two_open_streams_then_resume_first(&mut assembler, &config);

    assert_other_open_stream_event_id_rejected(assembler.apply(
        &config,
        LogicalEvent::Truncate {
            transaction_id: Some("43".to_string()),
            relations: vec![RelationId::new(16_384, "public", "sales")],
        },
    ));
    assert!(assembler.streamed.contains_key("42"));
    assert!(assembler.streamed.contains_key("43"));
}

#[test]
fn assembler_rejects_empty_stream_ddl_transaction_id() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    start_stream(&mut assembler, &config);

    assert_empty_stream_event_id_rejected(assembler.apply(
        &config,
        LogicalEvent::Ddl {
            transaction_id: Some(" ".to_string()),
            operation: DdlOperation::AddColumn,
            relation: RelationId::new(16_384, "public", "sales"),
            statement: "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"x\" text;".to_string(),
            schema_fingerprint_before: 12_345,
            schema_fingerprint_after: 67_890,
            target_auto_apply: true,
            release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
        },
    ));
    assert!(assembler.streamed.contains_key("42"));
}

#[test]
fn assembler_rejects_other_open_stream_ddl_transaction_id() {
    let mut assembler = TransactionAssembler::with_stream_spill_threshold(1);
    let config = assembler_config();

    start_two_open_streams_then_resume_first(&mut assembler, &config);

    assert_other_open_stream_event_id_rejected(assembler.apply(
        &config,
        LogicalEvent::Ddl {
            transaction_id: Some("43".to_string()),
            operation: DdlOperation::AddColumn,
            relation: RelationId::new(16_384, "public", "sales"),
            statement: "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"x\" text;".to_string(),
            schema_fingerprint_before: 12_345,
            schema_fingerprint_after: 67_890,
            target_auto_apply: true,
            release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
        },
    ));
    assert!(assembler.streamed.contains_key("42"));
    assert!(assembler.streamed.contains_key("43"));
}
