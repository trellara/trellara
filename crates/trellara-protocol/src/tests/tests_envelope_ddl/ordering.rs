use super::fixtures::{additive_ddl, ddl_envelope};
use super::*;

#[test]
fn checked_encoding_rejects_ddl_transaction_mismatch() {
    let envelope = ddl_envelope(
        "tx-1",
        vec![sample_change(2)],
        vec![additive_ddl("tx-other", 1)],
    );

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::DdlTransactionMismatch { .. })
    ));
}

#[test]
fn checked_encoding_rejects_ddl_order_collision_with_change() {
    let envelope = ddl_envelope(
        "tx-1",
        vec![sample_change(1)],
        vec![additive_ddl("tx-1", 1)],
    );

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::DuplicateTransactionEventOrder { total_order: 1 })
    ));
}

#[test]
fn checked_encoding_rejects_duplicate_ddl_event_order() {
    let envelope = ddl_envelope(
        "tx-1",
        Vec::new(),
        vec![additive_ddl("tx-1", 1), {
            let mut event = additive_ddl("tx-1", 1);
            event.statement =
                "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"customer_tier\" text;".to_string();
            event.schema_fingerprint_before = 67_890;
            event.schema_fingerprint_after = 98_765;
            event
        }],
    );

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::DuplicateTransactionEventOrder { total_order: 1 })
    ));
}
