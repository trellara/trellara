use serde::Serialize;

pub(crate) use crate::status_boundary_modes::*;

use crate::{
    status_boundary_eval::evaluate_transaction_boundary, transaction_boundary_evidence,
    FlowStatusSummary,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionBoundarySummary {
    pub(crate) mode: String,
    pub(crate) status: TransactionBoundaryStatus,
    pub(crate) guarantee: String,
    pub(crate) visibility_contract: String,
    pub(crate) parallel_replay_contract: String,
    pub(crate) evidence: String,
    pub(crate) source_checkpoint_durable: bool,
    pub(crate) target_checkpoint_caught_up: bool,
    pub(crate) manifest_barrier_required: bool,
    pub(crate) manifest_barrier_complete: Option<bool>,
    pub(crate) global_partition_watermark_caught_up: Option<bool>,
}

impl TransactionBoundarySummary {
    pub(crate) fn from_status(status: &FlowStatusSummary) -> Self {
        let evaluation = evaluate_transaction_boundary(status);

        Self {
            mode: status.mode.clone(),
            status: evaluation.status,
            guarantee: transaction_boundary_guarantee(&status.mode).to_string(),
            visibility_contract: transaction_visibility_contract(&status.mode).to_string(),
            parallel_replay_contract: parallel_replay_contract(&status.mode).to_string(),
            evidence: transaction_boundary_evidence(
                status,
                evaluation.manifest_barrier_required,
                evaluation.manifest_barrier_complete,
                evaluation.global_partition_watermark_caught_up,
            ),
            source_checkpoint_durable: evaluation.source_checkpoint_durable,
            target_checkpoint_caught_up: evaluation.target_checkpoint_caught_up,
            manifest_barrier_required: evaluation.manifest_barrier_required,
            manifest_barrier_complete: evaluation.manifest_barrier_complete,
            global_partition_watermark_caught_up: evaluation.global_partition_watermark_caught_up,
        }
    }
}

pub(crate) fn transaction_boundary_proof_evidence(boundary: &TransactionBoundarySummary) -> String {
    format!(
        "{}; {}; {}; {}",
        boundary.guarantee,
        boundary.visibility_contract,
        boundary.parallel_replay_contract,
        boundary.evidence
    )
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TransactionBoundaryStatus {
    Verified,
    PendingEvidence,
    AtRisk,
}
