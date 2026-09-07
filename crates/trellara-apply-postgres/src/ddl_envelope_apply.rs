use trellara_checkpoint::{DdlBarrierLookup, DdlBarrierStore, DdlBarrierSummary};
use trellara_protocol::{
    ddl_propagation_policy_sha256, summarize_ddl_propagation, TransactionEnvelope,
};

use crate::{
    record_target_ddl_barrier_from_envelope, target_ddl_dml_replay_proof,
    target_ddl_release_decision, ApplyError, ApplyOutcome, PostgresApplier, Result,
    TargetDdlAckEvidence, TargetDdlBarrierRequirements, TargetDdlDmlReplayProof,
    TargetDmlReleaseDecision,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlEnvelopeApplyOutcome {
    pub target_ack: TargetDdlAckEvidence,
    pub barrier_summary: DdlBarrierSummary,
    pub release_decision: TargetDmlReleaseDecision,
    pub ddl_propagation_decisions: String,
    pub ddl_target_ack_required: usize,
    pub ddl_propagation_policy_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlEnvelopeAndDmlApplyOutcome {
    pub ddl: Option<TargetDdlEnvelopeApplyOutcome>,
    pub dml: ApplyOutcome,
    pub ddl_dml_replay_proof: Option<TargetDdlDmlReplayProof>,
}

pub(crate) struct TargetDdlPropagationProof {
    pub(crate) decisions: String,
    pub(crate) target_ack_required: usize,
    pub(crate) policy_sha256: String,
}

pub async fn apply_target_ddl_envelope_then_dml<S: DdlBarrierStore + ?Sized>(
    applier: &mut PostgresApplier,
    store: &S,
    envelope: &TransactionEnvelope,
    requirements: TargetDdlBarrierRequirements,
) -> Result<TargetDdlEnvelopeAndDmlApplyOutcome> {
    let ddl =
        apply_target_ddl_envelope_and_record_ack(applier, store, envelope, requirements).await?;
    if let Some(ddl) = &ddl {
        ddl.release_decision
            .require_released_at_boundary(&envelope.commit_lsn)?;
    }
    let dml = applier
        .apply_envelope(&envelope.dml_replay_after_ddl_barrier())
        .await?;
    let ddl_dml_replay_proof = ddl
        .as_ref()
        .map(|ddl| target_ddl_dml_replay_proof(&ddl.target_ack, &ddl.release_decision, &dml))
        .transpose()?;
    Ok(TargetDdlEnvelopeAndDmlApplyOutcome {
        ddl,
        dml,
        ddl_dml_replay_proof,
    })
}

pub async fn apply_target_ddl_envelope_and_record_ack<S: DdlBarrierStore + ?Sized>(
    applier: &mut PostgresApplier,
    store: &S,
    envelope: &TransactionEnvelope,
    requirements: TargetDdlBarrierRequirements,
) -> Result<Option<TargetDdlEnvelopeApplyOutcome>> {
    let Some(recorded) =
        record_target_ddl_barrier_from_envelope(store, envelope, requirements).await?
    else {
        return Ok(None);
    };
    let target_ack = applier
        .apply_target_ddl_from_envelope_and_record_ack(
            envelope,
            store,
            recorded.schema_version.clone(),
        )
        .await?;
    let target_ack = require_target_ddl_ack(target_ack)?;
    let barrier_summary = require_ddl_barrier_summary(
        store,
        &DdlBarrierLookup::new(
            &envelope.source_id,
            &envelope.database_id,
            &envelope.dataset_id,
        ),
        &recorded.barrier_id,
    )
    .await?;
    let release_decision = target_ddl_release_decision(&barrier_summary);
    let propagation_proof = ddl_envelope_propagation_proof(envelope)?;
    Ok(Some(TargetDdlEnvelopeApplyOutcome {
        target_ack,
        barrier_summary,
        release_decision,
        ddl_propagation_decisions: propagation_proof.decisions,
        ddl_target_ack_required: propagation_proof.target_ack_required,
        ddl_propagation_policy_sha256: propagation_proof.policy_sha256,
    }))
}

pub(crate) fn ddl_envelope_propagation_proof(
    envelope: &TransactionEnvelope,
) -> Result<TargetDdlPropagationProof> {
    let summary = summarize_ddl_propagation(&envelope.ddl_events)?;
    Ok(TargetDdlPropagationProof {
        decisions: summary.evidence(),
        target_ack_required: summary.target_ack_required,
        policy_sha256: ddl_propagation_policy_sha256(&envelope.ddl_events)?,
    })
}

pub(crate) fn require_target_ddl_ack(
    target_ack: Option<TargetDdlAckEvidence>,
) -> Result<TargetDdlAckEvidence> {
    target_ack.ok_or(ApplyError::MissingDdlField {
        field: "target_ddl_ack",
    })
}

pub(crate) async fn require_ddl_barrier_summary<S: DdlBarrierStore + ?Sized>(
    store: &S,
    lookup: &DdlBarrierLookup,
    barrier_id: &str,
) -> Result<DdlBarrierSummary> {
    store
        .ddl_barrier_summary(lookup, barrier_id)
        .await?
        .ok_or(ApplyError::MissingDdlField {
            field: "ddl_barrier_summary",
        })
}
