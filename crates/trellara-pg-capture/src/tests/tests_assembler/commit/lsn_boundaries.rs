use super::helpers::begin_and_insert;
use super::*;

#[test]
fn assembler_rejects_empty_transaction_id_before_opening_transaction() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "  ".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id.trim().is_empty()
            && reason.contains("transaction_id")
            && reason.contains("empty")
    ));
}

#[test]
fn assembler_rejects_padded_transaction_id_before_opening_transaction() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: " tx-padded ".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == " tx-padded "
            && reason.contains("transaction_id")
            && reason.contains("surrounding whitespace")
    ));
}

#[test]
fn assembler_rejects_zero_begin_lsn_before_opening_transaction() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-zero-begin".to_string(),
                begin_lsn: "0/0".to_string(),
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == "tx-zero-begin" && reason.contains("begin_lsn")
    ));
}

#[test]
fn assembler_rejects_zero_commit_lsn_before_emitting_envelope() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    begin_and_insert(&mut assembler, &config, "tx-zero-commit", "0/16B6B00");

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/0".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == "tx-zero-commit" && reason.contains("commit_lsn")
    ));

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("valid commit after invalid boundary")
        .expect("envelope");

    assert_eq!(envelope.transaction_id, "tx-zero-commit");
    assert_eq!(envelope.changes.len(), 1);
}

#[test]
fn assembler_rejects_zero_commit_timestamp_before_emitting_envelope() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    begin_and_insert(&mut assembler, &config, "tx-zero-commit-ts", "0/16B6B00");

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 0,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == "tx-zero-commit-ts"
            && reason.contains("commit_timestamp_ms")
    ));

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("valid commit after invalid timestamp")
        .expect("envelope");

    assert_eq!(envelope.transaction_id, "tx-zero-commit-ts");
    assert_eq!(envelope.commit_timestamp_ms, 1_786_420_000_000);
}

#[test]
fn assembler_rejects_begin_lsn_after_commit_lsn_before_emitting_envelope() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    begin_and_insert(&mut assembler, &config, "tx-bad-order", "0/16B6C51");

    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        ),
        Err(CaptureError::InvalidTransactionBoundary {
            transaction_id,
            reason,
        }) if transaction_id == "tx-bad-order"
            && reason.contains("begin_lsn")
            && reason.contains("commit_lsn")
    ));
}

#[test]
fn assembler_canonicalizes_equivalent_lsn_text_before_envelope_emit() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    begin_and_insert(
        &mut assembler,
        &config,
        "tx-canonical-lsn",
        "00000000/016B6B00",
    );

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "00000000/016B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("commit")
        .expect("envelope");

    assert_eq!(envelope.begin_lsn, "0/16B6B00");
    assert_eq!(envelope.commit_lsn, "0/16B6C50");
    assert_eq!(
        envelope.changes[0].idempotency_key,
        "source-a:0/16B6C50:tx-canonical-lsn:1"
    );
}
