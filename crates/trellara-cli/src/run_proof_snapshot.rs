use std::path::Path;

use trellara_checkpoint::SnapshotRunState;

use crate::{
    snapshot_handoff_evidence, BootstrapSummary, RunProofGate, RunProofStatus, SnapshotCopySummary,
};

pub(crate) fn snapshot_handoff_boundary_run_proof(
    config_path: &Path,
    snapshot: Option<&SnapshotCopySummary>,
    bootstrap: &BootstrapSummary,
) -> RunProofGate {
    let config_display = config_path.display().to_string();
    let snapshot_handoff_state_verified = snapshot.is_some_and(|snapshot| {
        snapshot.state == SnapshotRunState::StreamHandoffReady.to_string()
            || snapshot.state == SnapshotRunState::Verified.to_string()
    });
    let snapshot_boundary_matches_bootstrap =
        snapshot.is_some_and(|snapshot| match bootstrap.consistent_lsn.as_ref() {
            Some(consistent_lsn) => consistent_lsn == &snapshot.consistent_lsn,
            None => true,
        });
    let status = if snapshot_handoff_state_verified && snapshot_boundary_matches_bootstrap {
        RunProofStatus::Verified
    } else if snapshot_handoff_state_verified && !snapshot_boundary_matches_bootstrap {
        RunProofStatus::AtRisk
    } else {
        RunProofStatus::NeedsEvidence
    };

    RunProofGate {
        code: "snapshot_handoff_boundary".to_string(),
        status,
        evidence: snapshot
            .map(|snapshot| {
                format!(
                    "{}; bootstrap_consistent_lsn={}",
                    snapshot_handoff_evidence(snapshot),
                    bootstrap.consistent_lsn.as_deref().unwrap_or("missing")
                )
            })
            .unwrap_or_else(|| {
                "snapshot was skipped; run needs explicit handoff evidence".to_string()
            }),
        proof_command: format!("trellara snapshot --config {config_display} --run-id <run-id>"),
    }
}
