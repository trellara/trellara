use trellara_checkpoint::{
    IcebergTableCommitIntent as CheckpointIcebergIntent,
    IcebergTableCommitReceipt as CheckpointIcebergReceipt,
    IcebergTableCommitStatus as CheckpointIcebergStatus,
};

use crate::{
    IcebergEpochCommitPlan, IcebergIntegrationError, IcebergTableAppendPlan,
    IcebergTableCommitReceipt, IcebergTableCommitStatus, Result,
};

pub fn iceberg_commit_intents_from_plan(
    plan: &IcebergEpochCommitPlan,
    planned_at: impl Into<String>,
) -> Vec<CheckpointIcebergIntent> {
    let planned_at = planned_at.into();
    plan.tables
        .iter()
        .map(|table| intent_from_table(plan, table, &planned_at))
        .collect()
}

pub fn iceberg_commit_receipt_checkpoint_row(
    plan: &IcebergEpochCommitPlan,
    receipt: &IcebergTableCommitReceipt,
    committed_at: impl Into<String>,
) -> Result<CheckpointIcebergReceipt> {
    let table = plan
        .tables
        .iter()
        .find(|table| {
            table.target == receipt.target && table.table_commit_id == receipt.table_commit_id
        })
        .ok_or_else(|| IcebergIntegrationError::UnplannedCommitReceipt {
            target: receipt.target.qualified_name(),
        })?;
    validate_receipt_field(
        table,
        "epoch_commit_id",
        &table.epoch_commit_id,
        &receipt.epoch_commit_id,
    )?;
    validate_receipt_field(
        table,
        "file_count",
        &table.file_count.to_string(),
        &receipt.file_count.to_string(),
    )?;
    validate_receipt_field(
        table,
        "record_count",
        &table.record_count.to_string(),
        &receipt.record_count.to_string(),
    )?;

    Ok(CheckpointIcebergReceipt {
        dataset_id: plan.dataset_id.clone(),
        epoch_id: plan.epoch_id.clone(),
        epoch_commit_id: receipt.epoch_commit_id.clone(),
        target: receipt.target.qualified_name(),
        table_commit_id: receipt.table_commit_id.clone(),
        snapshot_id: receipt.snapshot_id,
        file_count: receipt.file_count,
        record_count: receipt.record_count,
        status: checkpoint_status(receipt.status),
        committed_at: committed_at.into(),
    })
}

fn intent_from_table(
    plan: &IcebergEpochCommitPlan,
    table: &IcebergTableAppendPlan,
    planned_at: &str,
) -> CheckpointIcebergIntent {
    CheckpointIcebergIntent {
        dataset_id: plan.dataset_id.clone(),
        epoch_id: plan.epoch_id.clone(),
        epoch_commit_id: plan.epoch_commit_id.clone(),
        lake_table_name: table.lake_table_name.clone(),
        relation: table.relation.clone(),
        target: table.target.qualified_name(),
        table_commit_id: table.table_commit_id.clone(),
        manifest_digest: plan.manifest_digest.clone(),
        file_count: table.file_count,
        record_count: table.record_count,
        planned_at: planned_at.to_string(),
    }
}

fn validate_receipt_field(
    table: &IcebergTableAppendPlan,
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(IcebergIntegrationError::CommitReceiptMismatch {
            target: table.target.qualified_name(),
            field,
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}

fn checkpoint_status(status: IcebergTableCommitStatus) -> CheckpointIcebergStatus {
    match status {
        IcebergTableCommitStatus::Committed => CheckpointIcebergStatus::Committed,
        IcebergTableCommitStatus::AlreadyCommitted => CheckpointIcebergStatus::AlreadyCommitted,
    }
}
