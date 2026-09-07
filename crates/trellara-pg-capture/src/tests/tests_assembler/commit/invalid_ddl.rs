use super::*;

#[test]
fn assembler_rejects_auto_apply_ddl_without_post_ddl_release_gate_before_emit() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");

    assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-bad-ddl".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin");
    assembler
        .apply(
            &config,
            LogicalEvent::Ddl {
                transaction_id: None,
                operation: DdlOperation::AddColumn,
                relation,
                statement: "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;"
                    .to_string(),
                schema_fingerprint_before: 12_345,
                schema_fingerprint_after: 67_890,
                target_auto_apply: true,
                release_gate: "manual_release".to_string(),
            },
        )
        .expect("ddl");

    let error = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect_err("invalid DDL envelope");

    assert!(matches!(
        error,
        CaptureError::Protocol(trellara_protocol::ProtocolError::InvalidDdlEvent {
            total_order: 1,
            reason,
        }) if reason.contains("target_auto_apply DDL")
            && reason.contains(POST_DDL_DML_RELEASE_GATE)
    ));
}

#[test]
fn assembler_rejects_auto_apply_non_add_column_ddl_before_emit() {
    let mut assembler = TransactionAssembler::default();
    let config = assembler_config();
    let relation = RelationId::new(16_384, "public", "sales");

    assembler
        .apply(
            &config,
            LogicalEvent::Begin {
                transaction_id: "tx-bad-ddl-operation".to_string(),
                begin_lsn: "0/16B6B00".to_string(),
            },
        )
        .expect("begin");
    assembler
        .apply(
            &config,
            LogicalEvent::Ddl {
                transaction_id: None,
                operation: DdlOperation::RenameColumn,
                relation,
                statement: "ALTER TABLE \"public\".\"sales\" RENAME COLUMN \"coupon\" TO \"discount_code\";"
                    .to_string(),
                schema_fingerprint_before: 12_345,
                schema_fingerprint_after: 67_890,
                target_auto_apply: true,
                release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
            },
        )
        .expect("ddl");

    let error = assembler
        .apply(
            &config,
            LogicalEvent::Commit {
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect_err("invalid DDL envelope");

    assert!(matches!(
        error,
        CaptureError::Protocol(trellara_protocol::ProtocolError::InvalidDdlEvent {
            total_order: 1,
            reason,
        }) if reason.contains("target_auto_apply DDL")
            && reason.contains("operation=add_column")
    ));
}
