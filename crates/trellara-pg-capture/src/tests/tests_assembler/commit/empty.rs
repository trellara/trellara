use super::helpers::begin_and_insert;
use super::*;

#[test]
fn assembler_drops_empty_committed_transaction_and_consumes_boundary() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();

    assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-empty".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin");

    let emitted = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("empty commit");

    assert!(emitted.is_none());
    assert!(assembler.open.is_none());

    begin_and_insert(&mut assembler, &config, "tx-after-empty", "0/16B6D00");
    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6E00".to_string(),
                commit_timestamp_ms: 1_786_420_100_000,
            },
        )
        .expect("commit after empty transaction")
        .expect("envelope");

    assert_eq!(envelope.transaction_id, "tx-after-empty");
    assert_eq!(envelope.begin_lsn, "0/16B6D00");
    assert_eq!(envelope.commit_lsn, "0/16B6E00");
    assert_eq!(envelope.changes.len(), 1);
}

#[test]
fn assembler_drops_aborted_transaction_without_leaking_changes() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();

    begin_and_insert(&mut assembler, &config, "tx-rolled-back", "0/16B6B00");
    let emitted = assembler
        .apply(&config, LogicalEvent::Abort)
        .expect("abort rolled-back transaction");

    assert!(emitted.is_none());
    assert!(assembler.open.is_none());

    begin_and_insert(&mut assembler, &config, "tx-after-abort", "0/16B6D00");
    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6E00".to_string(),
                commit_timestamp_ms: 1_786_420_100_000,
            },
        )
        .expect("commit after abort")
        .expect("envelope");

    assert_eq!(envelope.transaction_id, "tx-after-abort");
    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(
        envelope.changes[0].idempotency_key,
        "source-a:0/16B6E00:tx-after-abort:1"
    );
}
