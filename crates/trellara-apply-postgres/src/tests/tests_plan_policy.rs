use super::*;
use std::collections::HashMap;

#[test]
fn insert_policy_omits_target_owned_columns() {
    let mut policies = HashMap::new();
    policies.insert(relation().display_name(), target_owned_policy());
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![insert_change()],
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.finalize_checksum();

    let statements = plan_envelope_with_policies(&envelope, &policies).expect("plan");

    assert_eq!(
        statements[0].sql,
        "insert into \"public\".\"sales\" (\"id\") values ($1)"
    );
    assert_eq!(statements[0].values.len(), 1);
    assert_eq!(statements[0].values[0].column, "id");
}

#[test]
fn update_policy_skips_target_owned_columns() {
    let mut policies = HashMap::new();
    policies.insert(relation().display_name(), target_owned_policy());
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![update_change()],
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.finalize_checksum();

    let statements = plan_envelope_with_policies(&envelope, &policies).expect("plan");

    assert!(statements.is_empty());
}

#[test]
fn update_policy_rejects_invalid_value_kind_before_skipping_target_owned_column() {
    let mut policies = HashMap::new();
    policies.insert(relation().display_name(), target_owned_policy());
    let mut change = update_change();
    change.after = Some(row(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue {
            name: "amount_cents".to_string(),
            type_oid: 20,
            value_kind: 99,
            text_value: "1499".to_string(),
            binary_value: Vec::new().into(),
            is_key: false,
        },
    ]));
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![change],
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.finalize_checksum();

    let error = plan_envelope_with_policies(&envelope, &policies).expect_err("invalid value kind");

    assert!(matches!(
        error,
        ApplyError::UnsupportedValueKind { column, value_kind: 99 } if column == "amount_cents"
    ));
}
