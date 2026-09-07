use super::*;

#[test]
fn identity_audit_marks_configured_primary_keys_ready_without_full() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("config");
    config.validate().expect("valid local config");

    let summary = IdentityAuditSummary::from_config(&config, Path::new("local.yml"));

    assert_eq!(summary.table_count, 1);
    assert_eq!(summary.pk_apply_ready_count, 1);
    assert_eq!(summary.live_identity_review_required_count, 0);
    assert!(summary.ordinary_pk_tables_do_not_require_full);
    assert!(summary
        .cdc_apply_gate
        .contains("released: configured primary-key apply"));
    let table = summary.tables.first().expect("identity audit table");
    assert_eq!(table.relation, "public.sales");
    assert_eq!(table.configured_primary_key.as_deref(), Some("id"));
    assert_eq!(table.status, IdentityAuditTableStatus::PkApplyReady);
    assert!(table
        .replica_identity_requirement
        .contains("REPLICA IDENTITY FULL is not required"));
    assert!(table
        .toast_handling
        .contains("absent non-key columns are treated as unchanged"));
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("plans_update_with_key_predicate")));
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("plans_delete_with_key_predicate")));
    assert!(
        summary
            .proof_commands
            .iter()
            .any(|command| command
                .contains("update_omits_absent_non_key_columns_for_unchanged_toast"))
    );
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("update_omits_explicit_unchanged_toast_marker")));
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("pgoutput_decoder_rejects_omitted_unchanged_key_column")));
    let guarantee_codes = summary
        .apply_guarantees
        .iter()
        .map(|guarantee| guarantee.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        guarantee_codes,
        vec![
            "default_identity_pk_apply",
            "key_predicate_update_delete",
            "unchanged_toast_preservation"
        ]
    );
    let default_identity = summary
        .apply_guarantees
        .iter()
        .find(|guarantee| guarantee.code == "default_identity_pk_apply")
        .expect("default identity guarantee");
    assert!(default_identity
        .guarantee
        .contains("advisory, not blockers"));
    assert!(default_identity
        .transaction_boundary
        .contains("before CDC apply begins"));
    let toast = summary
        .apply_guarantees
        .iter()
        .find(|guarantee| guarantee.code == "unchanged_toast_preservation")
        .expect("toast guarantee");
    assert!(toast.guarantee.contains("never written as null"));
    assert!(toast
        .transaction_boundary
        .contains("omit key columns fail closed"));
}

#[test]
fn identity_audit_requires_live_review_without_configured_primary_key() {
    let yaml = local_stream_yaml().replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n      verify:\n        primary_key: ''",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("config");

    let summary = IdentityAuditSummary::from_config(&config, Path::new("local.yml"));

    assert_eq!(summary.table_count, 1);
    assert_eq!(summary.pk_apply_ready_count, 0);
    assert_eq!(summary.live_identity_review_required_count, 1);
    assert!(!summary.ordinary_pk_tables_do_not_require_full);
    assert!(summary
        .cdc_apply_gate
        .contains("held: 1 table(s) require live primary-key"));
    let table = summary.tables.first().expect("identity audit table");
    assert_eq!(table.configured_primary_key, None);
    assert_eq!(
        table.status,
        IdentityAuditTableStatus::LiveIdentityReviewRequired
    );
    assert!(table
        .replica_identity_requirement
        .contains("REPLICA IDENTITY FULL or a replica identity index is required"));
}

#[test]
fn identity_audit_text_renders_structured_apply_guarantees() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("config");
    let summary = IdentityAuditSummary::from_config(&config, Path::new("local.yml"));

    let output = render_identity_audit_text(&summary);

    assert!(output.contains("apply_guarantees:"));
    assert!(output.contains("cdc_apply_gate: released: configured primary-key apply"));
    assert!(output.contains("- default_identity_pk_apply"));
    assert!(output.contains("transaction_boundary: accepted only when source-safety"));
    assert!(output.contains("- key_predicate_update_delete"));
    assert!(output.contains("- unchanged_toast_preservation"));
    assert!(output.contains("pgoutput_decoder_rejects_omitted_unchanged_key_column"));
}
