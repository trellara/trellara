use trellara_checkpoint::{DdlBarrier, DDL_BARRIER_CDC_TRANSACTION_BOUNDARY};
use trellara_protocol::{
    ddl_propagation_policy_sha256, summarize_ddl_propagation, TransactionBoundaryKey,
    TransactionEnvelope, DDL_PROPAGATION_CDC_BOUNDARY, POST_DDL_DML_RELEASE_GATE,
};

#[path = "ddl_protocol_schema.rs"]
mod ddl_protocol_schema;

use ddl_protocol_schema::{
    ddl_schema_version, final_ddl_event, validate_final_ddl_schema_version_evidence,
};

use crate::{
    collect_statement, ordered_ddl_events, Result, TargetDdlApplyPlan,
    TargetDdlBarrierRequirements, TARGET_DDL_TRANSACTION_BOUNDARY,
};

pub fn target_ddl_apply_plan_from_envelope(
    envelope: &TransactionEnvelope,
) -> Result<Option<TargetDdlApplyPlan>> {
    envelope.validate()?;
    if envelope.ddl_events.is_empty() {
        return Ok(None);
    }

    let mut blockers = Vec::new();
    let mut statements = Vec::new();
    for event in ordered_ddl_events(envelope) {
        collect_statement(event, &mut blockers, &mut statements);
    }

    Ok(Some(TargetDdlApplyPlan {
        barrier_id: ddl_barrier_id(envelope)?,
        transaction_boundary_rule: TARGET_DDL_TRANSACTION_BOUNDARY.to_string(),
        blockers,
        release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
        statements,
    }))
}

pub fn target_ddl_barrier_from_envelope(
    envelope: &TransactionEnvelope,
) -> Result<Option<DdlBarrier>> {
    target_ddl_barrier_from_envelope_with_requirements(
        envelope,
        TargetDdlBarrierRequirements::target_postgres_only(),
    )
}

pub fn target_ddl_barrier_from_envelope_with_requirements(
    envelope: &TransactionEnvelope,
    requirements: TargetDdlBarrierRequirements,
) -> Result<Option<DdlBarrier>> {
    envelope.validate()?;
    if envelope.ddl_events.is_empty() {
        return Ok(None);
    }
    let final_ddl_event = final_ddl_event(envelope)?;
    validate_final_ddl_schema_version_evidence(envelope, final_ddl_event)?;
    let boundary_key = boundary_key(envelope)?;

    Ok(Some(DdlBarrier {
        source_id: boundary_key.source_id.clone(),
        database_id: boundary_key.database_id.clone(),
        dataset_id: boundary_key.dataset_id.clone(),
        barrier_id: ddl_barrier_id_from_key(&boundary_key),
        barrier_lsn: boundary_key.commit_lsn,
        schema_version: ddl_schema_version(final_ddl_event),
        cdc_transaction_boundary: format!(
            "{DDL_BARRIER_CDC_TRANSACTION_BOUNDARY}; propagation_boundary={DDL_PROPAGATION_CDC_BOUNDARY}; {}; propagation_policy_sha256={}; DDL and DML share source transaction order; post-DDL DML stays invisible until required sink ACKs reach barrier_lsn",
            summarize_ddl_propagation(&envelope.ddl_events)?.evidence(),
            ddl_propagation_policy_sha256(&envelope.ddl_events)?
        ),
        required_sinks: requirements.required_sinks,
        requires_global_partition_pause: requirements.requires_global_partition_pause,
    }))
}

pub(crate) fn ddl_barrier_id(envelope: &TransactionEnvelope) -> Result<String> {
    Ok(ddl_barrier_id_from_key(&boundary_key(envelope)?))
}

fn ddl_barrier_id_from_key(boundary_key: &TransactionBoundaryKey) -> String {
    format!("{boundary_key}:ddl")
}

fn boundary_key(envelope: &TransactionEnvelope) -> Result<TransactionBoundaryKey> {
    Ok(envelope.boundary_key()?)
}
