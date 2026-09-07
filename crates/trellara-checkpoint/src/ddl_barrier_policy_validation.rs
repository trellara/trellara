use trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY;

use crate::{
    ddl_barrier_policy_evidence::{
        propagation_boundary_token, propagation_decision_tokens, propagation_policy_sha256_token,
    },
    CheckpointError, DdlBarrier, Result,
};

pub(crate) fn validate_propagation_policy_evidence(barrier: &DdlBarrier) -> Result<()> {
    if !barrier.cdc_transaction_boundary.contains("propagation_") {
        return Ok(());
    }

    require_canonical_propagation_boundary(barrier)?;
    require_complete_propagation_decisions(barrier)?;
    require_policy_digest(barrier)
}

fn require_canonical_propagation_boundary(barrier: &DdlBarrier) -> Result<()> {
    match propagation_boundary_token(&barrier.cdc_transaction_boundary).as_deref() {
        Some(DDL_PROPAGATION_CDC_BOUNDARY) => Ok(()),
        Some(_) => Err(invalid_policy_evidence(
            barrier,
            "propagation_boundary must use the canonical DDL propagation boundary",
        )),
        None => Err(invalid_policy_evidence(
            barrier,
            "propagation_boundary is required when propagation proof is present",
        )),
    }
}

fn require_complete_propagation_decisions(barrier: &DdlBarrier) -> Result<()> {
    let decisions = propagation_decision_tokens(&barrier.cdc_transaction_boundary);
    for prefix in [
        "auto_apply:",
        "manual_review:",
        "unsupported:",
        "target_ack_required:",
    ] {
        let Some(value) = decisions
            .iter()
            .find_map(|decision| decision.strip_prefix(prefix))
        else {
            return Err(invalid_policy_evidence(
                barrier,
                "propagation_decisions must include auto_apply, manual_review, unsupported, and target_ack_required counts",
            ));
        };
        if value.parse::<usize>().is_err() {
            return Err(invalid_policy_evidence(
                barrier,
                "propagation_decisions counts must be unsigned integers",
            ));
        }
    }
    Ok(())
}

fn require_policy_digest(barrier: &DdlBarrier) -> Result<()> {
    propagation_policy_sha256_token(&barrier.cdc_transaction_boundary)
        .map(|_| ())
        .ok_or_else(|| {
            invalid_policy_evidence(
                barrier,
                "propagation_policy_sha256 must be a 64-character SHA-256 hex digest",
            )
        })
}

fn invalid_policy_evidence(barrier: &DdlBarrier, reason: &str) -> CheckpointError {
    CheckpointError::Store(format!(
        "DDL barrier {} propagation policy evidence is invalid: {reason}",
        barrier.barrier_id
    ))
}
