use std::collections::BTreeMap;

use trellara_checkpoint::{
    IcebergTableCommitIntent as CheckpointIcebergIntent,
    IcebergTableCommitReceipt as CheckpointIcebergReceipt,
};

use crate::{IcebergEpochCommitPlan, IcebergIntegrationError, IcebergTableAppendPlan, Result};

pub(crate) fn persisted_intents_by_target(
    intents: &[CheckpointIcebergIntent],
) -> Result<BTreeMap<String, &CheckpointIcebergIntent>> {
    let mut by_target = BTreeMap::new();
    for intent in intents {
        if by_target.insert(intent.target.clone(), intent).is_some() {
            return Err(IcebergIntegrationError::ConflictingCatalogEvidence {
                target: intent.target.clone(),
                reason: "more than one checkpoint intent was provided for the target".to_string(),
            });
        }
    }
    Ok(by_target)
}

pub(crate) fn persisted_receipts_by_target(
    receipts: &[CheckpointIcebergReceipt],
) -> Result<BTreeMap<String, &CheckpointIcebergReceipt>> {
    let mut by_target = BTreeMap::new();
    for receipt in receipts {
        if by_target.insert(receipt.target.clone(), receipt).is_some() {
            return Err(IcebergIntegrationError::ConflictingCatalogEvidence {
                target: receipt.target.clone(),
                reason: "more than one checkpoint receipt was provided for the target".to_string(),
            });
        }
    }
    Ok(by_target)
}

pub(crate) fn validate_intent(
    table: &IcebergTableAppendPlan,
    expected: &CheckpointIcebergIntent,
    actual: &CheckpointIcebergIntent,
) -> Result<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(IcebergIntegrationError::ConflictingCatalogEvidence {
            target: table.target.qualified_name(),
            reason: "checkpoint intent does not match the current append plan".to_string(),
        })
    }
}

pub(crate) fn validate_receipt(
    plan: &IcebergEpochCommitPlan,
    table: &IcebergTableAppendPlan,
    receipt: &CheckpointIcebergReceipt,
) -> Result<()> {
    compare_receipt_field(table, "dataset_id", &plan.dataset_id, &receipt.dataset_id)?;
    compare_receipt_field(table, "epoch_id", &plan.epoch_id, &receipt.epoch_id)?;
    compare_receipt_field(
        table,
        "epoch_commit_id",
        &plan.epoch_commit_id,
        &receipt.epoch_commit_id,
    )?;
    compare_receipt_field(
        table,
        "table_commit_id",
        &table.table_commit_id,
        &receipt.table_commit_id,
    )?;
    compare_receipt_field(
        table,
        "file_count",
        &table.file_count.to_string(),
        &receipt.file_count.to_string(),
    )?;
    compare_receipt_field(
        table,
        "record_count",
        &table.record_count.to_string(),
        &receipt.record_count.to_string(),
    )
}

fn compare_receipt_field(
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
