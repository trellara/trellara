use serde::Serialize;
use trellara_apply_postgres::{target_ddl_release_decision, TargetDmlReleaseDecision};
use trellara_checkpoint::{DdlBarrier, DdlBarrierAck, DdlBarrierSummary};

#[path = "pilot_package_ddl_release/ack_commands.rs"]
mod ack_commands;
#[path = "pilot_package_ddl_release/evidence.rs"]
mod evidence;
#[path = "pilot_package_ddl_release/replay_proof.rs"]
mod replay_proof;

use crate::{
    config_source_database_id, pilot_package_ddl_release_acks::ddl_release_ack,
    DdlApplyPlanSummary, DdlPlanSummary, Result, TrellaraConfig,
};

use replay_proof::{ddl_dml_replay_proof, DdlDmlReplayProof};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlReleaseProof {
    pub(crate) source_id: String,
    pub(crate) database_id: String,
    pub(crate) dataset_id: String,
    pub(crate) barrier_id: String,
    pub(crate) barrier_lsn: String,
    pub(crate) schema_version: String,
    pub(crate) cdc_transaction_boundary: String,
    pub(crate) release_gate: String,
    pub(crate) required_sinks: Vec<String>,
    pub(crate) ack_commands: Vec<String>,
    pub(crate) ack_evidence: Vec<DdlBarrierAck>,
    pub(crate) release_evidence: Vec<String>,
    pub(crate) ddl_dml_replay_proof: DdlDmlReplayProof,
    pub(crate) release_summary: DdlBarrierSummary,
    pub(crate) release_decision: TargetDmlReleaseDecision,
}

pub(crate) fn pilot_package_ddl_release_proof(
    config: &TrellaraConfig,
    plan: &DdlPlanSummary,
    apply_plan: &DdlApplyPlanSummary,
) -> Result<DdlReleaseProof> {
    let barrier_lsn = "0/16B8000".to_string();
    let schema_version = "schema-v2".to_string();
    let required_sinks = plan
        .propagation
        .sinks
        .iter()
        .map(|sink| sink.name.clone())
        .collect::<Vec<_>>();
    let barrier = DdlBarrier {
        source_id: config.source.id.clone(),
        database_id: config_source_database_id(config),
        dataset_id: config.dataset.id.clone(),
        barrier_id: plan.propagation.barrier_id.clone(),
        barrier_lsn: barrier_lsn.clone(),
        schema_version: schema_version.clone(),
        cdc_transaction_boundary: plan.propagation.cdc_transaction_boundary.clone(),
        required_sinks: required_sinks.clone(),
        requires_global_partition_pause: plan.propagation.requires_global_partition_pause,
    };
    let ack_evidence = required_sinks
        .iter()
        .map(|sink| {
            ddl_release_ack(
                config,
                plan,
                apply_plan,
                sink,
                &barrier_lsn,
                &schema_version,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let ack_commands = required_sinks
        .iter()
        .map(|sink| {
            ack_commands::ddl_release_ack_command(
                config,
                plan,
                apply_plan,
                sink,
                &barrier_lsn,
                &schema_version,
            )
        })
        .collect();
    let release_summary =
        DdlBarrierSummary::try_from_barrier_and_acks(barrier.clone(), ack_evidence.clone())?;
    let release_decision = target_ddl_release_decision(&release_summary);
    let release_evidence = evidence::ddl_release_evidence(&release_summary);
    let ddl_dml_replay_proof = ddl_dml_replay_proof(&barrier, apply_plan);

    Ok(DdlReleaseProof {
        source_id: barrier.source_id,
        database_id: barrier.database_id,
        dataset_id: barrier.dataset_id,
        barrier_id: barrier.barrier_id,
        barrier_lsn,
        schema_version,
        cdc_transaction_boundary: barrier.cdc_transaction_boundary,
        release_gate: "post_ddl_dml_release".to_string(),
        required_sinks,
        ack_commands,
        ack_evidence,
        release_evidence,
        ddl_dml_replay_proof,
        release_summary,
        release_decision,
    })
}
