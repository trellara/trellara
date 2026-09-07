use std::collections::{BTreeMap, BTreeSet};

use crate::{
    IcebergEpochCommitPlan, IcebergEpochCommitSummary, IcebergIntegrationError,
    IcebergTableCommitReceipt, IcebergTableCommitStatus, Result,
};

pub fn iceberg_epoch_commit_summary(
    plan: &IcebergEpochCommitPlan,
    receipts: &[IcebergTableCommitReceipt],
) -> Result<IcebergEpochCommitSummary> {
    let expected = plan
        .tables
        .iter()
        .map(|table| (table.target.qualified_name(), table))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut snapshot_ids = BTreeMap::new();
    let mut newly_committed_table_count = 0usize;
    let mut already_committed_table_count = 0usize;

    for receipt in receipts {
        let target = receipt.target.qualified_name();
        if !seen.insert(target.clone()) {
            return Err(IcebergIntegrationError::DuplicateCommitReceipt { target });
        }
        let table = expected.get(&target).ok_or_else(|| {
            IcebergIntegrationError::UnplannedCommitReceipt {
                target: target.clone(),
            }
        })?;
        compare_receipt(
            &target,
            "epoch_commit_id",
            &table.epoch_commit_id,
            &receipt.epoch_commit_id,
        )?;
        compare_receipt(
            &target,
            "table_commit_id",
            &table.table_commit_id,
            &receipt.table_commit_id,
        )?;
        compare_receipt(
            &target,
            "file_count",
            &table.file_count.to_string(),
            &receipt.file_count.to_string(),
        )?;
        compare_receipt(
            &target,
            "record_count",
            &table.record_count.to_string(),
            &receipt.record_count.to_string(),
        )?;
        validate_receipt_snapshot_id(&target, receipt.snapshot_id)?;
        snapshot_ids.insert(target, receipt.snapshot_id);
        match receipt.status {
            IcebergTableCommitStatus::Committed => newly_committed_table_count += 1,
            IcebergTableCommitStatus::AlreadyCommitted => already_committed_table_count += 1,
        }
    }

    let missing_tables = expected
        .keys()
        .filter(|target| !seen.contains(*target))
        .cloned()
        .collect::<Vec<_>>();
    let ready_for_epoch_metadata = missing_tables.is_empty();

    Ok(IcebergEpochCommitSummary {
        dataset_id: plan.dataset_id.clone(),
        epoch_id: plan.epoch_id.clone(),
        epoch_commit_id: plan.epoch_commit_id.clone(),
        ready_for_epoch_metadata,
        expected_table_count: expected.len(),
        committed_table_count: receipts.len(),
        newly_committed_table_count,
        already_committed_table_count,
        missing_tables,
        snapshot_ids,
        epoch_snapshot_reference: ready_for_epoch_metadata
            .then(|| format!("trellara-iceberg-epoch:{}", plan.epoch_commit_id)),
    })
}

fn compare_receipt(target: &str, field: &'static str, expected: &str, actual: &str) -> Result<()> {
    if expected == actual {
        return Ok(());
    }
    Err(IcebergIntegrationError::CommitReceiptMismatch {
        target: target.to_string(),
        field,
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
}

fn validate_receipt_snapshot_id(target: &str, snapshot_id: i64) -> Result<()> {
    if snapshot_id > 0 {
        return Ok(());
    }
    Err(IcebergIntegrationError::InvalidCommitReceipt {
        target: target.to_string(),
        field: "snapshot_id",
        reason: "must be greater than zero before it can release _trellara_epochs".to_string(),
    })
}
