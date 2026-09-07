use super::*;

#[test]
fn checkpoint_intents_preserve_every_table_append_boundary() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");

    let intents = iceberg_commit_intents_from_plan(&plan, "planned-at");

    assert_eq!(intents.len(), plan.tables.len());
    for table in &plan.tables {
        let intent = intents
            .iter()
            .find(|intent| intent.table_commit_id == table.table_commit_id)
            .expect("intent");
        assert_eq!(intent.dataset_id, plan.dataset_id);
        assert_eq!(intent.epoch_id, plan.epoch_id);
        assert_eq!(intent.epoch_commit_id, plan.epoch_commit_id);
        assert_eq!(intent.target, table.target.qualified_name());
        assert_eq!(intent.file_count, table.file_count);
        assert_eq!(intent.record_count, table.record_count);
        assert_eq!(intent.planned_at, "planned-at");
    }
}

#[test]
fn checkpoint_receipt_rows_reject_mismatched_table_evidence() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let mut receipt = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);
    receipt.record_count += 1;

    let error = iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at")
        .expect_err("receipt mismatch");

    assert!(matches!(
        error,
        IcebergIntegrationError::CommitReceiptMismatch {
            field: "record_count",
            ..
        }
    ));
}

#[test]
fn checkpoint_receipt_rows_add_epoch_identity_for_recovery() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let receipt = receipt(
        &plan.tables[1],
        202,
        IcebergTableCommitStatus::AlreadyCommitted,
    );

    let row = iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at")
        .expect("checkpoint row");

    assert_eq!(row.dataset_id, plan.dataset_id);
    assert_eq!(row.epoch_id, plan.epoch_id);
    assert_eq!(row.epoch_commit_id, plan.epoch_commit_id);
    assert_eq!(row.target, plan.tables[1].target.qualified_name());
    assert_eq!(row.committed_at, "committed-at");
    assert_eq!(
        row.status,
        trellara_checkpoint::IcebergTableCommitStatus::AlreadyCommitted
    );
}
