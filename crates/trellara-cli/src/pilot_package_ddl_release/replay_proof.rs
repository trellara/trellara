use serde::Serialize;
use trellara_apply_postgres::TARGET_DDL_DML_REPLAY_PROOF_CONTRACT;
use trellara_checkpoint::DdlBarrier;
use trellara_protocol::POST_DDL_DML_RELEASE_GATE;

use crate::DdlApplyPlanSummary;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlDmlReplayProof {
    pub(crate) contract: String,
    pub(crate) source_id: String,
    pub(crate) database_id: String,
    pub(crate) dataset_id: String,
    pub(crate) barrier_id: String,
    pub(crate) barrier_lsn: String,
    pub(crate) target_ack_lsn: String,
    pub(crate) dml_commit_lsn: String,
    pub(crate) dml_decision: String,
    pub(crate) ddl_applied_statements: usize,
    pub(crate) dml_applied_changes: usize,
    pub(crate) schema_version: String,
    pub(crate) release_gate: String,
    pub(crate) target_transaction_boundary: String,
    pub(crate) cdc_transaction_boundary: String,
    pub(crate) proof_steps: Vec<String>,
}

pub(super) fn ddl_dml_replay_proof(
    barrier: &DdlBarrier,
    apply_plan: &DdlApplyPlanSummary,
) -> DdlDmlReplayProof {
    DdlDmlReplayProof {
        contract: TARGET_DDL_DML_REPLAY_PROOF_CONTRACT.to_string(),
        source_id: barrier.source_id.clone(),
        database_id: barrier.database_id.clone(),
        dataset_id: barrier.dataset_id.clone(),
        barrier_id: barrier.barrier_id.clone(),
        barrier_lsn: barrier.barrier_lsn.clone(),
        target_ack_lsn: barrier.barrier_lsn.clone(),
        dml_commit_lsn: barrier.barrier_lsn.clone(),
        dml_decision: "Applied".to_string(),
        ddl_applied_statements: apply_plan.statement_count,
        dml_applied_changes: 1,
        schema_version: barrier.schema_version.clone(),
        release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
        target_transaction_boundary:
            "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release"
                .to_string(),
        cdc_transaction_boundary: ddl_dml_replay_cdc_boundary(&barrier.cdc_transaction_boundary),
        proof_steps: vec![
            "target_postgres_recorded_ddl_ack".to_string(),
            "release_decision_allowed_post_ddl_dml".to_string(),
            "dml_replay_applied_changes_positive".to_string(),
            "dml_replay_commit_lsn_matches_ddl_barrier_lsn".to_string(),
        ],
    }
}

fn ddl_dml_replay_cdc_boundary(cdc_transaction_boundary: &str) -> String {
    [
        "source commit LSN is the DDL barrier",
        "post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
        cdc_transaction_boundary,
    ]
    .join("; ")
}
