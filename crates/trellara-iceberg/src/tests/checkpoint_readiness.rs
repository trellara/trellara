use super::*;
use trellara_lake::LakeCompletenessState;

#[test]
fn checkpoint_readiness_blocks_epoch_metadata_until_every_table_is_proven() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let first = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);
    let checkpoint_receipt =
        iceberg_commit_receipt_checkpoint_row(&plan, &first, "committed-at").expect("receipt");

    let report = verify_iceberg_epoch_checkpoint_readiness(&plan, &[checkpoint_receipt])
        .expect("readiness report");

    assert!(!report.ready_for_epoch_metadata);
    assert_eq!(report.expected_table_count, 2);
    assert_eq!(report.proven_table_count, 1);
    assert_eq!(report.missing_tables.len(), 1);
    assert_eq!(report.tables.len(), 2);
    assert!(report.epoch_snapshot_reference.is_none());
    assert!(report.tables.iter().any(|table| {
        table.status == IcebergTableReadinessStatus::MissingCheckpointReceipt
            && !table.checkpoint_proven
            && table.snapshot_id.is_none()
    }));
}

#[test]
fn checkpoint_readiness_reports_epoch_reference_when_all_tables_are_proven() {
    let mut raw_plan = raw_cdc_plan();
    raw_plan.epoch_metadata.epoch_row.state = LakeCompletenessState::CompleteWithGaps;
    raw_plan.epoch_metadata.epoch_row.policy = "publish_with_gaps".to_string();
    raw_plan.epoch_metadata.epoch_row.complete_source_count = 1;
    raw_plan.epoch_metadata.epoch_row.missing_source_count = 1;
    let plan = plan_iceberg_epoch_commit(
        &raw_plan,
        &commit_config().accepting_complete_with_gaps(),
        completed_files(),
    )
    .expect("plan");
    let receipts = plan
        .tables
        .iter()
        .enumerate()
        .map(|(index, table)| {
            let receipt = receipt(
                table,
                i64::try_from(index + 1).expect("snapshot id"),
                IcebergTableCommitStatus::Committed,
            );
            iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at")
                .expect("receipt row")
        })
        .collect::<Vec<_>>();

    let report =
        verify_iceberg_epoch_checkpoint_readiness(&plan, &receipts).expect("readiness report");

    assert!(report.ready_for_epoch_metadata);
    assert!(report.accepted_complete_with_gaps);
    assert_eq!(report.proven_table_count, report.expected_table_count);
    assert!(report.missing_tables.is_empty());
    assert!(report.epoch_snapshot_reference.is_some());
    assert!(report.tables.iter().all(|table| {
        table.status == IcebergTableReadinessStatus::Ready
            && table.checkpoint_proven
            && table.snapshot_id.is_some()
    }));
}

#[test]
fn checkpoint_readiness_rejects_conflicting_checkpoint_receipts() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let receipt = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);
    let mut checkpoint_receipt =
        iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at").expect("receipt");
    checkpoint_receipt.epoch_commit_id = "wrong".to_string();

    let error = verify_iceberg_epoch_checkpoint_readiness(&plan, &[checkpoint_receipt])
        .expect_err("readiness should reject mismatched receipt");

    assert!(matches!(
        error,
        IcebergIntegrationError::CommitReceiptMismatch {
            field: "epoch_commit_id",
            ..
        }
    ));
}
