use super::fixtures::{additive_ddl, ddl_envelope};
use super::*;

#[test]
fn checked_encoding_round_trips_additive_ddl_event() {
    let envelope = ddl_envelope(
        "tx-1",
        vec![sample_change(2)],
        vec![additive_ddl("tx-1", 1)],
    );

    let encoded = envelope.encode_checked().expect("encode DDL envelope");
    let decoded = TransactionEnvelope::decode_checked(&encoded).expect("decode DDL envelope");

    assert_eq!(decoded.ddl_events.len(), 1);
    assert_eq!(decoded.ddl_events[0].total_order, 1);
    assert_eq!(
        DdlOperation::try_from(decoded.ddl_events[0].operation).expect("operation"),
        DdlOperation::AddColumn
    );
    assert_eq!(
        decoded.ddl_events[0].release_gate,
        POST_DDL_DML_RELEASE_GATE
    );
}

#[test]
fn checked_encoding_round_trips_manual_review_ddl_with_release_gate() {
    let envelope = ddl_envelope(
        "tx-rename",
        Vec::new(),
        vec![DdlEvent::manual_review(
            "tx-rename",
            1,
            DdlOperation::RenameColumn,
            RelationId::new(16_384, "public", "sales"),
            "ALTER TABLE \"public\".\"sales\" RENAME COLUMN \"coupon\" TO \"discount_code\";",
            12_345,
            67_890,
        )],
    );

    let encoded = envelope.encode_checked().expect("encode DDL envelope");
    let decoded = TransactionEnvelope::decode_checked(&encoded).expect("decode DDL envelope");

    assert_eq!(decoded.ddl_events.len(), 1);
    assert_eq!(
        DdlOperation::try_from(decoded.ddl_events[0].operation).expect("operation"),
        DdlOperation::RenameColumn
    );
    assert!(!decoded.ddl_events[0].target_auto_apply);
    assert_eq!(
        decoded.ddl_events[0].release_gate,
        POST_DDL_DML_RELEASE_GATE
    );
}
