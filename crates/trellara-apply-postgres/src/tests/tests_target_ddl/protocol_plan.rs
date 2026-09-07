use super::*;

#[test]
fn target_ddl_plan_from_envelope_preserves_transaction_boundary() {
    let envelope = ddl_envelope();
    let apply_plan = target_ddl_apply_plan_from_envelope(&envelope)
        .expect("DDL plan")
        .expect("DDL events");
    let transaction_plan = plan_target_ddl_transaction(apply_plan).expect("target DDL transaction");

    assert_eq!(
        transaction_plan.barrier_id,
        "source:retail:sales:tx-ddl:0/16B6C50:ddl"
    );
    assert_eq!(transaction_plan.statement_count, 1);
    assert_eq!(
        transaction_plan.statements[0].sql,
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;"
    );
    assert_eq!(transaction_plan.statements[0].operation, "ddl");
}

#[test]
fn target_ddl_plan_from_envelope_orders_statements_by_source_total_order() {
    let mut envelope = ddl_envelope();
    envelope.ddl_events = vec![
        DdlEvent::additive_column(
            "tx-ddl",
            2,
            relation(),
            "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"audit_note\" text;",
            67_890,
            98_765,
        ),
        DdlEvent::additive_column(
            "tx-ddl",
            1,
            relation(),
            "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
            12_345,
            67_890,
        ),
    ];
    envelope.finalize_checksum();

    let apply_plan = target_ddl_apply_plan_from_envelope(&envelope)
        .expect("DDL plan")
        .expect("DDL events");
    let transaction_plan = plan_target_ddl_transaction(apply_plan).expect("target DDL transaction");

    assert_eq!(transaction_plan.statement_count, 2);
    assert_eq!(
        transaction_plan
            .statements
            .iter()
            .map(|statement| statement.sql.as_str())
            .collect::<Vec<_>>(),
        vec![
            "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
            "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"audit_note\" text;",
        ]
    );
}

#[test]
fn target_ddl_plan_from_envelope_returns_none_without_ddl_events() {
    assert!(
        target_ddl_apply_plan_from_envelope(&envelope("tx-1", "0/16B6C50"))
            .expect("plan")
            .is_none()
    );
}

#[test]
fn target_ddl_plan_from_envelope_blocks_manual_review_events() {
    let mut envelope = ddl_envelope();
    envelope.ddl_events[0].target_auto_apply = false;
    envelope.finalize_checksum();

    let apply_plan = target_ddl_apply_plan_from_envelope(&envelope)
        .expect("DDL plan")
        .expect("DDL events");

    assert!(matches!(
        plan_target_ddl_transaction(apply_plan),
        Err(ApplyError::DdlApplyBlocked { blockers, .. })
            if blockers[0].contains("operator review")
    ));
}

#[test]
fn target_ddl_plan_from_envelope_rejects_auto_apply_non_additive_events() {
    let mut envelope = ddl_envelope();
    envelope.ddl_events[0].operation = DdlOperation::RenameColumn as i32;
    envelope.finalize_checksum();

    assert!(matches!(
        target_ddl_apply_plan_from_envelope(&envelope),
        Err(ApplyError::Protocol(trellara_protocol::ProtocolError::InvalidDdlEvent {
            total_order: 1,
            reason,
        })) if reason.contains("target_auto_apply DDL")
            && reason.contains("operation=add_column")
    ));
}

#[test]
fn target_ddl_plan_from_envelope_blocks_statement_relation_mismatch() {
    let mut envelope = ddl_envelope();
    envelope.ddl_events[0].statement =
        "ALTER TABLE \"public\".\"orders\" ADD COLUMN \"discount_code\" text;".to_string();
    envelope.finalize_checksum();

    let apply_plan = target_ddl_apply_plan_from_envelope(&envelope)
        .expect("DDL plan")
        .expect("DDL events");

    assert!(matches!(
        plan_target_ddl_transaction(apply_plan),
        Err(ApplyError::DdlApplyBlocked { blockers, .. })
            if blockers[0].contains("statement relation does not match DDL event relation")
                && blockers[0].contains("public.sales")
                && blockers[0].contains("public\".\"orders")
    ));
}

#[test]
fn target_ddl_plan_from_envelope_preserves_manual_review_classification() {
    let mut envelope = ddl_envelope();
    envelope.ddl_events[0] = DdlEvent::manual_review(
        "tx-ddl",
        1,
        DdlOperation::ChangePartitionKey,
        RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"store_id\" TYPE bigint;",
        12_345,
        67_890,
    );
    envelope.finalize_checksum();

    let apply_plan = target_ddl_apply_plan_from_envelope(&envelope)
        .expect("DDL plan")
        .expect("DDL events");

    assert_eq!(
        apply_plan.barrier_id,
        "source:retail:sales:tx-ddl:0/16B6C50:ddl"
    );
    assert!(apply_plan.statements.is_empty());
    assert!(apply_plan.blockers[0].contains("operator approval"));
    assert!(apply_plan.blockers[0].contains("post-DDL DML release"));
}
