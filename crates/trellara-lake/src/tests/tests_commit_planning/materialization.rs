use super::*;

#[test]
fn strict_commit_plans_raw_current_and_scd2_operations() {
    let envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    let plan = plan_commit(&envelope, &config()).expect("plan");

    assert_eq!(
        plan.visibility_boundary,
        LakeVisibilityBoundary::StrictEnvelope
    );
    assert_eq!(plan.operation_count, 3);
    assert_eq!(
        plan.operations
            .iter()
            .map(|operation| (operation.materialization, operation.write_kind))
            .collect::<Vec<_>>(),
        vec![
            (LakeMaterialization::RawCdc, LakeWriteKind::AppendEvent),
            (
                LakeMaterialization::CurrentState,
                LakeWriteKind::UpsertCurrent
            ),
            (
                LakeMaterialization::Scd2History,
                LakeWriteKind::InsertVersion
            ),
        ]
    );
    assert!(plan
        .operations
        .iter()
        .all(|operation| operation.record_key.as_deref() == Some("sale-1")));
    assert!(plan.operations[0]
        .row
        .iter()
        .all(|column| column.name != "updated_at"));
}

#[test]
fn update_closes_previous_scd2_version_and_inserts_new_version() {
    let envelope = envelope(vec![change(
        Operation::Update,
        1,
        Some(row("sale-1", "10", "2026-01-01")),
        Some(row("sale-1", "12", "2026-01-02")),
    )]);
    let plan = plan_commit(&envelope, &config()).expect("plan");

    assert_eq!(
        plan.operations
            .iter()
            .filter(|operation| operation.materialization == LakeMaterialization::Scd2History)
            .map(|operation| operation.write_kind)
            .collect::<Vec<_>>(),
        vec![LakeWriteKind::CloseVersion, LakeWriteKind::InsertVersion]
    );
}

#[test]
fn raw_cdc_preserves_unchanged_toast_marker() {
    let after = RowImage::new(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::text("amount", 25, "12", false),
        ColumnValue::unchanged_toast("receipt_blob", 17, false),
    ]);
    let envelope = envelope(vec![change(
        Operation::Update,
        1,
        Some(row("sale-1", "10", "2026-01-01")),
        Some(after),
    )]);

    let plan = plan_commit(&envelope, &config()).expect("plan");
    let raw = plan
        .operations
        .iter()
        .find(|operation| operation.materialization == LakeMaterialization::RawCdc)
        .expect("raw CDC operation");

    assert!(raw.row.iter().any(|column| {
        column.name == "receipt_blob" && column.value == LakeValue::UnchangedToast
    }));
}

#[test]
fn delete_removes_current_row_and_closes_history_version() {
    let envelope = envelope(vec![change(
        Operation::Delete,
        1,
        Some(row("sale-1", "10", "2026-01-01")),
        None,
    )]);
    let plan = plan_commit(&envelope, &config()).expect("plan");

    assert!(plan.operations.iter().any(|operation| {
        operation.materialization == LakeMaterialization::CurrentState
            && operation.write_kind == LakeWriteKind::DeleteCurrent
    }));
    assert!(plan.operations.iter().any(|operation| {
        operation.materialization == LakeMaterialization::Scd2History
            && operation.write_kind == LakeWriteKind::CloseVersion
    }));
}
