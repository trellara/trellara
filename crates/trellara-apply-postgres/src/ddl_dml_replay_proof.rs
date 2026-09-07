use serde::Serialize;

use crate::{
    ddl_dml_replay_proof_validation::{
        require_fresh_dml_replay_applied, require_matching_ack_barrier_lsn,
        require_matching_ack_identity, require_matching_release_gate, require_same_lsn,
        require_valid_target_ack_evidence,
    },
    ApplyOutcome, Result, TargetDdlAckEvidence, TargetDmlReleaseDecision,
    TARGET_DDL_TRANSACTION_BOUNDARY,
};

pub const TARGET_DDL_DML_REPLAY_PROOF_CONTRACT: &str =
    "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TargetDdlDmlReplayProof {
    pub contract: &'static str,
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub barrier_lsn: String,
    pub target_ack_lsn: String,
    pub dml_commit_lsn: String,
    pub schema_version: String,
    pub ddl_applied_statements: usize,
    pub dml_applied_changes: usize,
    pub dml_decision: String,
    pub release_gate: String,
    pub target_transaction_boundary: String,
    pub cdc_transaction_boundary: String,
    pub proof_steps: Vec<&'static str>,
}

pub(crate) fn target_ddl_dml_replay_proof(
    target_ack: &TargetDdlAckEvidence,
    release_decision: &TargetDmlReleaseDecision,
    dml: &ApplyOutcome,
) -> Result<TargetDdlDmlReplayProof> {
    release_decision.require_released_at_boundary(&dml.commit_lsn)?;
    require_valid_target_ack_evidence(target_ack)?;
    require_fresh_dml_replay_applied(dml, release_decision)?;
    require_matching_ack_identity(target_ack, release_decision)?;
    require_matching_release_gate(target_ack, release_decision)?;
    require_matching_ack_barrier_lsn(target_ack, release_decision)?;
    require_same_lsn(
        "target ACK",
        &target_ack.ack_lsn,
        &release_decision.barrier_lsn,
        &release_decision.barrier_id,
    )?;
    require_same_lsn(
        "DML replay",
        &dml.commit_lsn,
        &release_decision.barrier_lsn,
        &release_decision.barrier_id,
    )?;

    Ok(TargetDdlDmlReplayProof {
        contract: TARGET_DDL_DML_REPLAY_PROOF_CONTRACT,
        source_id: release_decision.source_id.clone(),
        database_id: release_decision.database_id.clone(),
        dataset_id: release_decision.dataset_id.clone(),
        barrier_id: release_decision.barrier_id.clone(),
        barrier_lsn: release_decision.barrier_lsn.clone(),
        target_ack_lsn: target_ack.ack_lsn.clone(),
        dml_commit_lsn: dml.commit_lsn.clone(),
        schema_version: target_ack.schema_version.clone(),
        ddl_applied_statements: target_ack.applied_statements,
        dml_applied_changes: dml.applied_changes,
        dml_decision: format!("{:?}", dml.decision),
        release_gate: release_decision.release_gate.clone(),
        target_transaction_boundary: TARGET_DDL_TRANSACTION_BOUNDARY.to_string(),
        cdc_transaction_boundary: release_decision.cdc_transaction_boundary.clone(),
        proof_steps: vec![
            "target_postgres_recorded_ddl_ack",
            "release_decision_allowed_post_ddl_dml",
            "dml_replay_applied_changes_positive",
            "dml_replay_commit_lsn_matches_ddl_barrier_lsn",
        ],
    })
}
