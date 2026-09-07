use std::future::Future;

use trellara_checkpoint::{IcebergCommitStore, IcebergTableCommitIntent};

use crate::checkpoint_bridge::{
    iceberg_commit_intents_from_plan, iceberg_commit_receipt_checkpoint_row,
};
use crate::{
    IcebergEpochCommitPlan, IcebergIntegrationError, IcebergTableAppendPlan,
    IcebergTableCommitReceipt, IcebergTableCommitStatus, Result,
};

pub async fn commit_iceberg_epoch_with_checkpoint<S, F, Fut>(
    store: &S,
    plan: &IcebergEpochCommitPlan,
    planned_at: impl Into<String>,
    committed_at: impl Into<String>,
    mut commit_table: F,
) -> Result<Vec<IcebergTableCommitReceipt>>
where
    S: IcebergCommitStore,
    F: FnMut(IcebergTableAppendPlan) -> Fut,
    Fut: Future<Output = Result<IcebergTableCommitReceipt>>,
{
    let planned_at = planned_at.into();
    let committed_at = committed_at.into();
    let intents = iceberg_commit_intents_from_plan(plan, planned_at);
    let mut receipts = Vec::with_capacity(plan.tables.len());

    for (table, intent) in plan.tables.iter().zip(intents) {
        if let Some(receipt) = load_existing_receipt(store, plan, table, &intent).await? {
            receipts.push(receipt);
            continue;
        }
        record_intent(store, intent).await?;
        let receipt = commit_table(table.clone()).await?;
        let checkpoint_receipt =
            iceberg_commit_receipt_checkpoint_row(plan, &receipt, committed_at.clone())?;
        store
            .record_iceberg_commit_receipt(checkpoint_receipt)
            .await
            .map_err(|error| checkpoint_error("record_iceberg_commit_receipt", error))?;
        receipts.push(receipt);
    }

    Ok(receipts)
}

async fn load_existing_receipt<S>(
    store: &S,
    plan: &IcebergEpochCommitPlan,
    table: &IcebergTableAppendPlan,
    intent: &IcebergTableCommitIntent,
) -> Result<Option<IcebergTableCommitReceipt>>
where
    S: IcebergCommitStore,
{
    let Some(receipt) = store
        .load_iceberg_commit_receipt(&intent.key())
        .await
        .map_err(|error| checkpoint_error("load_iceberg_commit_receipt", error))?
    else {
        return Ok(None);
    };
    validate_checkpoint_receipt(plan, table, &receipt)?;
    Ok(Some(IcebergTableCommitReceipt {
        target: table.target.clone(),
        epoch_commit_id: receipt.epoch_commit_id,
        table_commit_id: receipt.table_commit_id,
        snapshot_id: receipt.snapshot_id,
        file_count: receipt.file_count,
        record_count: receipt.record_count,
        status: IcebergTableCommitStatus::AlreadyCommitted,
    }))
}

async fn record_intent<S>(store: &S, intent: IcebergTableCommitIntent) -> Result<()>
where
    S: IcebergCommitStore,
{
    store
        .record_iceberg_commit_intent(intent)
        .await
        .map_err(|error| checkpoint_error("record_iceberg_commit_intent", error))
}

fn validate_checkpoint_receipt(
    plan: &IcebergEpochCommitPlan,
    table: &IcebergTableAppendPlan,
    receipt: &trellara_checkpoint::IcebergTableCommitReceipt,
) -> Result<()> {
    compare("dataset_id", &plan.dataset_id, &receipt.dataset_id, table)?;
    compare("epoch_id", &plan.epoch_id, &receipt.epoch_id, table)?;
    compare(
        "epoch_commit_id",
        &plan.epoch_commit_id,
        &receipt.epoch_commit_id,
        table,
    )?;
    compare(
        "table_commit_id",
        &table.table_commit_id,
        &receipt.table_commit_id,
        table,
    )?;
    compare(
        "file_count",
        &table.file_count.to_string(),
        &receipt.file_count.to_string(),
        table,
    )?;
    compare(
        "record_count",
        &table.record_count.to_string(),
        &receipt.record_count.to_string(),
        table,
    )
}

fn compare(
    field: &'static str,
    expected: &str,
    actual: &str,
    table: &IcebergTableAppendPlan,
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

fn checkpoint_error(
    operation: &'static str,
    error: trellara_checkpoint::CheckpointError,
) -> IcebergIntegrationError {
    IcebergIntegrationError::Checkpoint {
        operation,
        message: error.to_string(),
    }
}
