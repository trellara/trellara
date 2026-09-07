use trellara_checkpoint::parse_lsn;

use crate::{
    ddl_digest::sha256_is_valid, ApplyDecision, ApplyError, ApplyOutcome, Result,
    TargetDdlAckEvidence, TargetDmlReleaseDecision,
};

pub(crate) fn require_valid_target_ack_evidence(target_ack: &TargetDdlAckEvidence) -> Result<()> {
    if target_ack.applied_statements == 0 {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "applied_statements",
            reason: "target ACK must prove at least one applied DDL statement".to_string(),
        });
    }
    if !sha256_is_valid(&target_ack.plan_sha256) {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "plan_sha256",
            reason: "must be a 64-character SHA-256 hex digest".to_string(),
        });
    }
    if target_ack.statement_sha256s.len() != target_ack.applied_statements {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "statement_sha256",
            reason: format!(
                "expected {} statement digests",
                target_ack.applied_statements
            ),
        });
    }
    if !target_ack
        .statement_sha256s
        .iter()
        .all(|digest| sha256_is_valid(digest))
    {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "statement_sha256",
            reason: "every statement digest must be a 64-character SHA-256 hex digest".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn require_fresh_dml_replay_applied(
    dml: &ApplyOutcome,
    release_decision: &TargetDmlReleaseDecision,
) -> Result<()> {
    if dml.decision == ApplyDecision::Applied && dml.applied_changes > 0 {
        return Ok(());
    }
    Err(ApplyError::DdlDmlReleaseBlocked {
        barrier_id: release_decision.barrier_id.clone(),
        blockers: vec![format!(
            "post-DDL DML replay must apply at least one change before proof release; decision={:?} applied_changes={}",
            dml.decision, dml.applied_changes
        )],
    })
}

pub(crate) fn require_matching_ack_identity(
    target_ack: &TargetDdlAckEvidence,
    release_decision: &TargetDmlReleaseDecision,
) -> Result<()> {
    require_same_field(
        "source_id",
        &target_ack.source_id,
        &release_decision.source_id,
        &release_decision.barrier_id,
    )?;
    require_same_field(
        "dataset_id",
        &target_ack.dataset_id,
        &release_decision.dataset_id,
        &release_decision.barrier_id,
    )?;
    require_same_field(
        "database_id",
        &target_ack.database_id,
        &release_decision.database_id,
        &release_decision.barrier_id,
    )?;
    require_same_field(
        "barrier_id",
        &target_ack.barrier_id,
        &release_decision.barrier_id,
        &release_decision.barrier_id,
    )
}

pub(crate) fn require_matching_release_gate(
    target_ack: &TargetDdlAckEvidence,
    release_decision: &TargetDmlReleaseDecision,
) -> Result<()> {
    if target_ack.sink == "target_postgres"
        && target_ack.release_gate == release_decision.release_gate
    {
        return Ok(());
    }
    Err(ApplyError::DdlDmlReleaseBlocked {
        barrier_id: release_decision.barrier_id.clone(),
        blockers: vec![format!(
            "target ACK sink {} with release gate {} does not match release decision gate {}",
            target_ack.sink, target_ack.release_gate, release_decision.release_gate
        )],
    })
}

pub(crate) fn require_matching_ack_barrier_lsn(
    target_ack: &TargetDdlAckEvidence,
    release_decision: &TargetDmlReleaseDecision,
) -> Result<()> {
    let Some(ack_barrier_lsn) = &target_ack.barrier_lsn else {
        return Err(ApplyError::DdlDmlReleaseBlocked {
            barrier_id: release_decision.barrier_id.clone(),
            blockers: vec!["target ACK is missing canonical barrier_lsn evidence".to_string()],
        });
    };
    require_same_lsn(
        "target ACK barrier",
        ack_barrier_lsn,
        &release_decision.barrier_lsn,
        &release_decision.barrier_id,
    )
}

fn require_same_field(
    field: &'static str,
    actual: &str,
    expected: &str,
    barrier_id: &str,
) -> Result<()> {
    if actual == expected {
        return Ok(());
    }
    Err(ApplyError::DdlDmlReleaseBlocked {
        barrier_id: barrier_id.to_string(),
        blockers: vec![format!(
            "target ACK {field} {actual} does not match release decision {field} {expected}"
        )],
    })
}

pub(crate) fn require_same_lsn(
    label: &'static str,
    actual: &str,
    barrier_lsn: &str,
    barrier_id: &str,
) -> Result<()> {
    if parse_lsn(actual) == parse_lsn(barrier_lsn) {
        return Ok(());
    }
    Err(ApplyError::DdlDmlReleaseBlocked {
        barrier_id: barrier_id.to_string(),
        blockers: vec![format!(
            "{label} LSN {actual} does not match barrier LSN {barrier_lsn}"
        )],
    })
}
