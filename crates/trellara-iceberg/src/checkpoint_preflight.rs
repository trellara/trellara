use trellara_checkpoint::{
    IcebergTableCommitIntent as CheckpointIcebergIntent,
    IcebergTableCommitReceipt as CheckpointIcebergReceipt,
};

use crate::checkpoint_bridge::iceberg_commit_intents_from_plan;
use crate::checkpoint_preflight_evidence::{
    persisted_intents_by_target, persisted_receipts_by_target, validate_intent, validate_receipt,
};
use crate::{
    IcebergEpochCommitPlan, IcebergPreflightAction, IcebergPreflightDecision,
    IcebergTableAppendPlan, Result,
};

pub fn plan_iceberg_checkpoint_preflight(
    plan: &IcebergEpochCommitPlan,
    persisted_intents: &[CheckpointIcebergIntent],
    persisted_receipts: &[CheckpointIcebergReceipt],
    planned_at: impl Into<String>,
) -> Result<Vec<IcebergPreflightDecision>> {
    let expected_intents = iceberg_commit_intents_from_plan(plan, planned_at);
    let persisted_intents = persisted_intents_by_target(persisted_intents)?;
    let persisted_receipts = persisted_receipts_by_target(persisted_receipts)?;

    plan.tables
        .iter()
        .zip(expected_intents)
        .map(|(table, expected_intent)| {
            let receipt = persisted_receipts.get(&table.target.qualified_name());
            let intent = persisted_intents.get(&table.target.qualified_name());
            decision_for_table(
                plan,
                table,
                &expected_intent,
                intent.copied(),
                receipt.copied(),
            )
        })
        .collect()
}

fn decision_for_table(
    plan: &IcebergEpochCommitPlan,
    table: &IcebergTableAppendPlan,
    expected_intent: &CheckpointIcebergIntent,
    persisted_intent: Option<&CheckpointIcebergIntent>,
    persisted_receipt: Option<&CheckpointIcebergReceipt>,
) -> Result<IcebergPreflightDecision> {
    if let Some(receipt) = persisted_receipt {
        validate_receipt(plan, table, receipt)?;
        return Ok(decision(
            table,
            IcebergPreflightAction::SkipAlreadyCommitted,
            "matching checkpoint receipt proves this table commit already reached the catalog",
        ));
    }

    if let Some(intent) = persisted_intent {
        validate_intent(table, expected_intent, intent)?;
        return Ok(decision(
            table,
            IcebergPreflightAction::CommitAfterRecordedIntent,
            "matching checkpoint intent exists but no validated receipt has been recorded",
        ));
    }

    Ok(decision(
        table,
        IcebergPreflightAction::RecordIntentThenCommit,
        "no checkpoint intent exists for this deterministic table commit",
    ))
}

fn decision(
    table: &IcebergTableAppendPlan,
    action: IcebergPreflightAction,
    reason: &str,
) -> IcebergPreflightDecision {
    IcebergPreflightDecision {
        target: table.target.clone(),
        table_commit_id: table.table_commit_id.clone(),
        action,
        reason: reason.to_string(),
    }
}
