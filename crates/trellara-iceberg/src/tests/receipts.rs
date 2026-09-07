use super::*;

#[test]
fn epoch_visibility_waits_for_every_table_receipt() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let first_receipt = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);

    let pending =
        iceberg_epoch_commit_summary(&plan, std::slice::from_ref(&first_receipt)).expect("pending");
    assert!(!pending.ready_for_epoch_metadata);
    assert_eq!(pending.committed_table_count, 1);
    assert_eq!(pending.missing_tables.len(), 1);
    assert!(pending.epoch_snapshot_reference.is_none());

    let second_receipt = receipt(
        &plan.tables[1],
        202,
        IcebergTableCommitStatus::AlreadyCommitted,
    );
    let ready =
        iceberg_epoch_commit_summary(&plan, &[first_receipt, second_receipt]).expect("ready");
    assert!(ready.ready_for_epoch_metadata);
    assert_eq!(ready.newly_committed_table_count, 1);
    assert_eq!(ready.already_committed_table_count, 1);
    assert_eq!(ready.snapshot_ids.len(), 2);
    assert!(ready.epoch_snapshot_reference.is_some());
}

#[test]
fn conflicting_or_duplicate_receipts_fail_closed() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let receipt = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);
    assert!(matches!(
        iceberg_epoch_commit_summary(&plan, &[receipt.clone(), receipt.clone()]),
        Err(IcebergIntegrationError::DuplicateCommitReceipt { .. })
    ));

    let mut conflicting = receipt;
    conflicting.table_commit_id = "wrong".to_string();
    assert!(matches!(
        iceberg_epoch_commit_summary(&plan, &[conflicting]),
        Err(IcebergIntegrationError::CommitReceiptMismatch {
            field: "table_commit_id",
            ..
        })
    ));
}

#[test]
fn receipt_snapshot_ids_must_be_real_iceberg_snapshots() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let receipt = receipt(&plan.tables[0], 0, IcebergTableCommitStatus::Committed);

    assert!(matches!(
        iceberg_epoch_commit_summary(&plan, &[receipt]),
        Err(IcebergIntegrationError::InvalidCommitReceipt {
            field: "snapshot_id",
            ..
        })
    ));
}
