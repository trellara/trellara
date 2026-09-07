use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use trellara_checkpoint::{
    IcebergCommitStore, IcebergTableCommitReceipt as CheckpointIcebergReceipt,
};

use crate::epoch_readiness_evidence::{checkpoint_receipt_to_catalog_receipt, table_readiness};
use crate::{
    iceberg_epoch_commit_summary, IcebergEpochCommitPlan, IcebergIntegrationError, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergEpochReadinessReport {
    pub dataset_id: String,
    pub epoch_id: String,
    pub epoch_commit_id: String,
    pub ready_for_epoch_metadata: bool,
    pub accepted_complete_with_gaps: bool,
    pub epoch_visibility_rule: String,
    pub expected_table_count: usize,
    pub proven_table_count: usize,
    pub missing_tables: Vec<String>,
    pub snapshot_ids: BTreeMap<String, i64>,
    pub epoch_snapshot_reference: Option<String>,
    pub tables: Vec<IcebergTableReadinessEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableReadinessEvidence {
    pub target: String,
    pub table_commit_id: String,
    pub status: IcebergTableReadinessStatus,
    pub checkpoint_proven: bool,
    pub snapshot_id: Option<i64>,
    pub reason: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergTableReadinessStatus {
    Ready,
    MissingCheckpointReceipt,
}

pub fn verify_iceberg_epoch_checkpoint_readiness(
    plan: &IcebergEpochCommitPlan,
    checkpoint_receipts: &[CheckpointIcebergReceipt],
) -> Result<IcebergEpochReadinessReport> {
    let receipts = checkpoint_receipts
        .iter()
        .map(|receipt| checkpoint_receipt_to_catalog_receipt(plan, receipt))
        .collect::<Result<Vec<_>>>()?;
    let summary = iceberg_epoch_commit_summary(plan, &receipts)?;
    let by_target = receipts
        .iter()
        .map(|receipt| (receipt.target.qualified_name(), receipt))
        .collect::<BTreeMap<_, _>>();
    let tables = plan
        .tables
        .iter()
        .map(|table| {
            let target = table.target.qualified_name();
            table_readiness(
                &target,
                &table.table_commit_id,
                by_target.get(&target).copied(),
            )
        })
        .collect::<Vec<_>>();

    Ok(IcebergEpochReadinessReport {
        dataset_id: plan.dataset_id.clone(),
        epoch_id: plan.epoch_id.clone(),
        epoch_commit_id: plan.epoch_commit_id.clone(),
        ready_for_epoch_metadata: summary.ready_for_epoch_metadata,
        accepted_complete_with_gaps: plan.accepted_complete_with_gaps,
        epoch_visibility_rule: plan.epoch_visibility_rule.clone(),
        expected_table_count: summary.expected_table_count,
        proven_table_count: summary.committed_table_count,
        missing_tables: summary.missing_tables,
        snapshot_ids: summary.snapshot_ids,
        epoch_snapshot_reference: summary.epoch_snapshot_reference,
        tables,
    })
}

pub async fn verify_iceberg_epoch_checkpoint_store_readiness<S>(
    store: &S,
    plan: &IcebergEpochCommitPlan,
) -> Result<IcebergEpochReadinessReport>
where
    S: IcebergCommitStore,
{
    let receipts = store
        .list_iceberg_commit_receipts_for_epoch(&plan.dataset_id, &plan.epoch_id)
        .await
        .map_err(|error| IcebergIntegrationError::Checkpoint {
            operation: "list_iceberg_commit_receipts_for_epoch",
            message: error.to_string(),
        })?;
    verify_iceberg_epoch_checkpoint_readiness(plan, &receipts)
}
