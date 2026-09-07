use super::*;

#[test]
fn update_delete_zero_row_match_fails_closed() {
    let update = plan_change(&update_change()).expect("update plan");
    let error = validate_affected_rows(&update, 0).expect_err("zero-row update");

    assert!(matches!(
        error,
        ApplyError::NoRowsMatched {
            total_order: 2,
            operation: "update",
        }
    ));

    let delete = plan_change(&delete_change()).expect("delete plan");
    let error = validate_affected_rows(&delete, 0).expect_err("zero-row delete");

    assert!(matches!(
        error,
        ApplyError::NoRowsMatched {
            total_order: 3,
            operation: "delete",
        }
    ));
}

#[test]
fn inserts_and_truncates_do_not_require_row_match() {
    let insert = plan_change(&insert_change()).expect("insert plan");
    validate_affected_rows(&insert, 0).expect("insert row count");

    let truncate = plan_change(&truncate_change()).expect("truncate plan");
    validate_affected_rows(&truncate, 0).expect("truncate row count");
}

#[test]
fn truncate_without_relation_fails_closed() {
    let mut change = truncate_change();
    change.relation = None;

    assert!(matches!(
        plan_change(&change),
        Err(ApplyError::MissingRelation { total_order: 4 })
    ));
}

#[test]
fn envelope_without_schema_version_evidence_fails_closed() {
    let mut envelope = envelope("tx-missing-schema-evidence", "0/16B6C50");
    envelope.schema_versions.clear();
    envelope.finalize_checksum();

    assert!(matches!(
        plan_envelope(&envelope),
        Err(ApplyError::MissingSchemaVersionEvidence {
            total_order: 1,
            relation,
        }) if relation == "public.sales"
    ));
}

#[test]
fn update_without_key_fails_closed() {
    let mut change = update_change();
    change.before = Some(row(vec![ColumnValue::text("id", 23, "sale-1", false)]));

    assert!(matches!(
        plan_change(&change),
        Err(ApplyError::MissingKeyColumns { total_order: 2 })
    ));
}

#[test]
fn update_with_unchanged_toast_key_fails_closed() {
    let mut change = update_change();
    change.before = Some(row(vec![ColumnValue::unchanged_toast("id", 23, true)]));

    assert!(matches!(
        plan_change(&change),
        Err(ApplyError::UnchangedToastKeyColumn { total_order: 2, column }) if column == "id"
    ));
}

#[test]
fn update_without_before_and_unchanged_toast_key_fails_before_mutability_check() {
    let mut change = update_change();
    change.before = None;
    change.after = Some(row(vec![ColumnValue::unchanged_toast("id", 23, true)]));

    assert!(matches!(
        plan_change(&change),
        Err(ApplyError::UnchangedToastKeyColumn { total_order: 2, column }) if column == "id"
    ));
}

#[test]
fn update_without_before_and_without_key_fails_before_mutability_check() {
    let mut change = update_change();
    change.before = None;
    change.after = Some(row(vec![ColumnValue::unchanged_toast(
        "receipt_blob",
        25,
        false,
    )]));

    assert!(matches!(
        plan_change(&change),
        Err(ApplyError::MissingKeyColumns { total_order: 2 })
    ));
}

#[test]
fn delete_with_unchanged_toast_key_fails_closed() {
    let mut change = delete_change();
    change.before = Some(row(vec![ColumnValue::unchanged_toast("id", 23, true)]));

    assert!(matches!(
        plan_change(&change),
        Err(ApplyError::UnchangedToastKeyColumn { total_order: 3, column }) if column == "id"
    ));
}

#[test]
fn update_without_mutable_columns_fails_closed() {
    let mut change = update_change();
    change.after = Some(row(vec![ColumnValue::text("id", 23, "sale-1", true)]));

    assert!(matches!(
        plan_change(&change),
        Err(ApplyError::NoMutableColumns { total_order: 2 })
    ));
}
