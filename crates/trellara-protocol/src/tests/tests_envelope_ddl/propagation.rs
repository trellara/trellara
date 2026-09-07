use super::fixtures::additive_ddl;
use super::*;

#[test]
fn propagation_classifies_additive_auto_apply_with_cdc_boundary() {
    let event = additive_ddl("tx-1", 1);

    let decision = classify_ddl_propagation(&event).expect("DDL propagation decision");

    assert_eq!(decision.operation, DdlOperation::AddColumn);
    assert_eq!(decision.disposition, DdlPropagationDisposition::AutoApply);
    assert!(decision.can_auto_apply());
    assert!(decision.requires_target_ack);
    assert_eq!(decision.release_gate, POST_DDL_DML_RELEASE_GATE);
    assert_eq!(decision.cdc_boundary, DDL_PROPAGATION_CDC_BOUNDARY);
}

#[test]
fn propagation_classifies_operator_reviewed_additive_ddl() {
    let mut event = additive_ddl("tx-1", 1);
    event.target_auto_apply = false;
    event.release_gate.clear();

    let decision = classify_ddl_propagation(&event).expect("DDL propagation decision");

    assert_eq!(decision.operation, DdlOperation::AddColumn);
    assert_eq!(
        decision.disposition,
        DdlPropagationDisposition::ManualReview
    );
    assert!(!decision.can_auto_apply());
    assert!(decision.requires_target_ack);
    assert!(decision.reason.contains("operator review"));
}

#[test]
fn propagation_classifies_opaque_ddl_as_unsupported() {
    let mut event = additive_ddl("tx-1", 1);
    event.operation = DdlOperation::Other as i32;
    event.target_auto_apply = false;

    let decision = classify_ddl_propagation(&event).expect("DDL propagation decision");

    assert_eq!(decision.operation, DdlOperation::Other);
    assert_eq!(decision.disposition, DdlPropagationDisposition::Unsupported);
    assert!(!decision.can_auto_apply());
    assert!(!decision.requires_target_ack);
}

#[test]
fn default_source_policy_auto_applies_only_additive_columns() {
    assert!(default_target_auto_apply_for_ddl(DdlOperation::AddColumn));
    assert!(!default_target_auto_apply_for_ddl(
        DdlOperation::RenameColumn
    ));
    assert!(!default_target_auto_apply_for_ddl(
        DdlOperation::ChangePartitionKey
    ));
    assert!(!default_target_auto_apply_for_ddl(DdlOperation::Other));
}

#[test]
fn default_release_gate_marks_reviewable_ddl_boundaries() {
    assert_eq!(
        default_release_gate_for_ddl(DdlOperation::AddColumn),
        POST_DDL_DML_RELEASE_GATE
    );
    assert_eq!(
        default_release_gate_for_ddl(DdlOperation::RenameColumn),
        POST_DDL_DML_RELEASE_GATE
    );
    assert_eq!(default_release_gate_for_ddl(DdlOperation::Other), "");
}

#[test]
fn propagation_policy_digest_is_stable_for_event_order() {
    let first = additive_ddl("tx-1", 2);
    let second = DdlEvent::manual_review(
        "tx-1",
        1,
        DdlOperation::ChangePartitionKey,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"store_id\" TYPE bigint;",
        11,
        12,
    );

    let forward = ddl_propagation_policy_sha256(&[first.clone(), second.clone()]).expect("digest");
    let reverse = ddl_propagation_policy_sha256(&[second, first]).expect("digest");

    assert_eq!(forward, reverse);
    assert_eq!(forward.len(), 64);
}

#[test]
fn propagation_policy_row_carries_release_gate_and_cdc_boundary() {
    let event = additive_ddl("tx-1", 1);
    let decision = classify_ddl_propagation(&event).expect("DDL propagation decision");

    let row = crate::ddl_propagation_policy::ddl_policy_row(&event, &decision);

    assert!(row.contains(POST_DDL_DML_RELEASE_GATE));
    assert!(row.contains(DDL_PROPAGATION_CDC_BOUNDARY));
    assert!(row.contains("|auto_apply|true|true|post_ddl_dml_release|post_ddl_dml_release|"));
}

#[test]
fn propagation_policy_digest_rejects_invalid_release_gate() {
    let mut event = additive_ddl("tx-1", 1);
    event.release_gate = "manual_release".to_string();

    assert!(matches!(
        ddl_propagation_policy_sha256(&[event]),
        Err(ProtocolError::InvalidDdlEvent { reason, .. })
            if reason.contains("reviewable DDL")
    ));
}

#[test]
fn propagation_policy_row_changes_when_release_gate_policy_changes() {
    let event = additive_ddl("tx-1", 1);
    let mut decision = classify_ddl_propagation(&event).expect("DDL propagation decision");
    let normal_row = crate::ddl_propagation_policy::ddl_policy_row(&event, &decision);

    decision.release_gate = "future_release_gate";
    let changed_row = crate::ddl_propagation_policy::ddl_policy_row(&event, &decision);

    assert_ne!(normal_row, changed_row);
    assert!(changed_row.contains("future_release_gate"));
}

#[test]
fn propagation_policy_row_changes_when_cdc_boundary_policy_changes() {
    let event = additive_ddl("tx-1", 1);
    let mut decision = classify_ddl_propagation(&event).expect("DDL propagation decision");
    let normal_row = crate::ddl_propagation_policy::ddl_policy_row(&event, &decision);

    decision.cdc_boundary = "future_cdc_boundary";
    let changed_row = crate::ddl_propagation_policy::ddl_policy_row(&event, &decision);

    assert_ne!(normal_row, changed_row);
    assert!(changed_row.contains("future_cdc_boundary"));
}

#[test]
fn propagation_policy_digest_changes_when_ack_requirement_changes() {
    let auto_apply = ddl_propagation_policy_sha256(&[additive_ddl("tx-1", 1)]).expect("digest");

    let unsupported = ddl_propagation_policy_sha256(&[DdlEvent::manual_review(
        "tx-1",
        1,
        DdlOperation::Other,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"coupon_code\" SET STORAGE EXTERNAL;",
        67_890,
        67_891,
    )])
    .expect("digest");

    assert_ne!(auto_apply, unsupported);
}

#[test]
fn propagation_summary_counts_decisions_and_target_ack_requirements() {
    let events = vec![
        additive_ddl("tx-1", 1),
        DdlEvent::manual_review(
            "tx-1",
            2,
            DdlOperation::RenameColumn,
            RelationId::new(16_384, "public", "sales"),
            "ALTER TABLE \"public\".\"sales\" RENAME COLUMN \"discount_code\" TO \"coupon_code\";",
            67_890,
            67_891,
        ),
        DdlEvent::manual_review(
            "tx-1",
            3,
            DdlOperation::Other,
            RelationId::new(16_384, "public", "sales"),
            "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"coupon_code\" SET STORAGE EXTERNAL;",
            67_891,
            67_892,
        ),
    ];

    let summary = summarize_ddl_propagation(&events).expect("summary");

    assert_eq!(summary.auto_apply, 1);
    assert_eq!(summary.manual_review, 1);
    assert_eq!(summary.unsupported, 1);
    assert_eq!(summary.target_ack_required, 2);
    assert_eq!(
        summary.evidence(),
        "propagation_decisions=auto_apply:1,manual_review:1,unsupported:1,target_ack_required:2"
    );
}

#[test]
fn propagation_summary_rejects_reviewable_ddl_without_release_gate() {
    let mut event = DdlEvent::manual_review(
        "tx-1",
        1,
        DdlOperation::RenameColumn,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" RENAME COLUMN \"discount_code\" TO \"coupon_code\";",
        67_890,
        67_891,
    );
    event.release_gate.clear();

    assert!(matches!(
        summarize_ddl_propagation(&[event]),
        Err(ProtocolError::InvalidDdlEvent { reason, .. })
            if reason.contains("reviewable DDL")
    ));
}

#[test]
fn propagation_summary_rejects_unsupported_ddl_with_release_gate() {
    let mut event = DdlEvent::manual_review(
        "tx-1",
        1,
        DdlOperation::Other,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"coupon_code\" SET STORAGE EXTERNAL;",
        67_890,
        67_891,
    );
    event.release_gate = "manual_gate".to_string();

    assert!(matches!(
        summarize_ddl_propagation(&[event]),
        Err(ProtocolError::InvalidDdlEvent { reason, .. })
            if reason.contains("unsupported DDL")
    ));
}
