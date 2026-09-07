use std::path::Path;

use serde::Serialize;

use crate::{
    bounded_large_transaction_run_proof, convergence_verification_run_proof,
    local_durability_ack_run_proof, snapshot_handoff_boundary_run_proof,
    source_bootstrap_position_run_proof, target_apply_checkpoint_run_proof, BootstrapSummary,
    RelaySummary, SnapshotCopySummary, TrellaraConfig, VerifySummary,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RunProofGate {
    pub(crate) code: String,
    pub(crate) status: RunProofStatus,
    pub(crate) evidence: String,
    pub(crate) proof_command: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RunProofStatus {
    Verified,
    NeedsEvidence,
    AtRisk,
}

pub(crate) fn run_proof_chain(
    config_path: &Path,
    config: &TrellaraConfig,
    snapshot: Option<&SnapshotCopySummary>,
    bootstrap: &BootstrapSummary,
    relay: &RelaySummary,
    apply: &crate::ApplySummary,
    verify: Option<&VerifySummary>,
) -> Vec<RunProofGate> {
    vec![
        snapshot_handoff_boundary_run_proof(config_path, snapshot, bootstrap),
        source_bootstrap_position_run_proof(config_path, bootstrap),
        bounded_large_transaction_run_proof(config_path, config),
        local_durability_ack_run_proof(config_path, relay),
        target_apply_checkpoint_run_proof(config_path, apply),
        convergence_verification_run_proof(config_path, verify),
    ]
}

pub(crate) fn run_proof_status_label(status: RunProofStatus) -> &'static str {
    match status {
        RunProofStatus::Verified => "verified",
        RunProofStatus::NeedsEvidence => "needs_evidence",
        RunProofStatus::AtRisk => "at_risk",
    }
}
