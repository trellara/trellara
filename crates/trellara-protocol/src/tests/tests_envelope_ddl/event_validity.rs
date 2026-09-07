use super::fixtures::{additive_ddl, ddl_envelope};
use super::*;

#[test]
fn checked_encoding_rejects_ddl_without_schema_fingerprint_change() {
    let mut event = additive_ddl("tx-1", 1);
    event.schema_fingerprint_after = event.schema_fingerprint_before;
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![event]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidDdlEvent { reason, .. })
            if reason.contains("schema fingerprints")
    ));
}

#[test]
fn checked_encoding_rejects_ddl_without_relation() {
    let mut event = additive_ddl("tx-1", 1);
    event.relation = None;
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![event]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::MissingDdlEventField {
            total_order: 1,
            field: "relation"
        })
    ));
}

#[test]
fn checked_encoding_rejects_zero_ddl_total_order() {
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![additive_ddl("tx-1", 0)]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidDdlEvent { reason, .. })
            if reason.contains("total_order")
    ));
}

#[test]
fn checked_encoding_rejects_auto_apply_ddl_without_post_ddl_release_gate() {
    let mut event = additive_ddl("tx-1", 1);
    event.release_gate = "manual_release".to_string();
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![event]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidDdlEvent { reason, .. })
            if reason.contains("target_auto_apply DDL")
                && reason.contains(POST_DDL_DML_RELEASE_GATE)
    ));
}

#[test]
fn checked_encoding_rejects_auto_apply_non_add_column_ddl() {
    let mut event = additive_ddl("tx-1", 1);
    event.operation = DdlOperation::RenameColumn as i32;
    event.statement =
        "ALTER TABLE \"public\".\"sales\" RENAME COLUMN \"coupon\" TO \"discount_code\";"
            .to_string();
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![event]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidDdlEvent { reason, .. })
            if reason.contains("target_auto_apply DDL")
                && reason.contains("operation=add_column")
    ));
}

#[test]
fn checked_encoding_rejects_reviewable_ddl_without_post_ddl_release_gate() {
    let mut event = DdlEvent::manual_review(
        "tx-1",
        1,
        DdlOperation::RenameColumn,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" RENAME COLUMN \"coupon\" TO \"discount_code\";",
        12_345,
        67_890,
    );
    event.release_gate.clear();
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![event]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::MissingDdlEventField {
            total_order: 1,
            field: "release_gate"
        })
    ));
}

#[test]
fn checked_encoding_allows_unsupported_ddl_without_release_gate() {
    let event = DdlEvent::manual_review(
        "tx-1",
        1,
        DdlOperation::Other,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"coupon\" SET STORAGE EXTERNAL;",
        12_345,
        67_890,
    );
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![event]);

    envelope
        .encode_checked()
        .expect("unsupported DDL can be captured as blocked evidence");
}

#[test]
fn checked_encoding_rejects_unsupported_ddl_operation() {
    let mut event = additive_ddl("tx-1", 1);
    event.operation = 99;
    let envelope = ddl_envelope("tx-1", Vec::new(), vec![event]);

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::UnsupportedDdlOperation {
            total_order: 1,
            operation: 99,
        })
    ));
}
