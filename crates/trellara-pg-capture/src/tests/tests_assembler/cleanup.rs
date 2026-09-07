use super::*;
use trellara_protocol::{Operation, ReplicaIdentity};

#[test]
fn assembler_drops_aborted_transaction() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");

    assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-1".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin");
    assembler
        .apply(
            &config,
            LogicalEvent::Change {
                transaction_id: None,
                relation,
                operation: Operation::Insert,
                replica_identity: ReplicaIdentity::Default,
                before: None,
                after: None,
            },
        )
        .expect("change");

    assert!(assembler
        .apply(&config, LogicalEvent::Abort)
        .expect("abort")
        .is_none());
    assert!(matches!(
        assembler.apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        ),
        Err(CaptureError::CommitWithoutBegin)
    ));
}

#[test]
fn assembler_drops_empty_committed_transaction() {
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

    assert!(assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("commit")
        .is_none());
}
