use trellara_checkpoint::IcebergTableCommitReceipt as CheckpointIcebergReceipt;

use crate::epoch_readiness::{IcebergTableReadinessEvidence, IcebergTableReadinessStatus};
use crate::{
    IcebergEpochCommitPlan, IcebergIntegrationError, IcebergTableCommitReceipt,
    IcebergTableCommitStatus, Result,
};

pub(crate) fn checkpoint_receipt_to_catalog_receipt(
    plan: &IcebergEpochCommitPlan,
    receipt: &CheckpointIcebergReceipt,
) -> Result<IcebergTableCommitReceipt> {
    let table = plan
        .tables
        .iter()
        .find(|table| table.target.qualified_name() == receipt.target)
        .ok_or_else(|| IcebergIntegrationError::UnplannedCommitReceipt {
            target: receipt.target.clone(),
        })?;

    Ok(IcebergTableCommitReceipt {
        target: table.target.clone(),
        epoch_commit_id: receipt.epoch_commit_id.clone(),
        table_commit_id: receipt.table_commit_id.clone(),
        snapshot_id: receipt.snapshot_id,
        file_count: receipt.file_count,
        record_count: receipt.record_count,
        status: match receipt.status {
            trellara_checkpoint::IcebergTableCommitStatus::Committed => {
                IcebergTableCommitStatus::Committed
            }
            trellara_checkpoint::IcebergTableCommitStatus::AlreadyCommitted => {
                IcebergTableCommitStatus::AlreadyCommitted
            }
        },
    })
}

pub(crate) fn table_readiness(
    target: &str,
    table_commit_id: &str,
    receipt: Option<&IcebergTableCommitReceipt>,
) -> IcebergTableReadinessEvidence {
    match receipt {
        Some(receipt) => IcebergTableReadinessEvidence {
            target: target.to_string(),
            table_commit_id: table_commit_id.to_string(),
            status: IcebergTableReadinessStatus::Ready,
            checkpoint_proven: true,
            snapshot_id: Some(receipt.snapshot_id),
            reason: "checkpoint receipt proves this table append reached Iceberg".to_string(),
        },
        None => IcebergTableReadinessEvidence {
            target: target.to_string(),
            table_commit_id: table_commit_id.to_string(),
            status: IcebergTableReadinessStatus::MissingCheckpointReceipt,
            checkpoint_proven: false,
            snapshot_id: None,
            reason:
                "epoch metadata is blocked until this table append records a checkpoint receipt"
                    .to_string(),
        },
    }
}
