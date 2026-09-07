use serde::{Deserialize, Serialize};

use crate::{
    ddl_barrier_ack_identity::validate_ack_boundaries,
    ddl_barrier_blocker_details::ddl_release_blocker_details,
    ddl_barrier_blockers::{
        ddl_release_blocker_codes, ddl_release_blockers, DdlBarrierBlockerContext,
    },
    ddl_barrier_policy_evidence::{
        propagation_boundary_token, propagation_decision_tokens, propagation_policy_sha256_token,
    },
    ddl_barrier_release_actions::ddl_barrier_release_actions,
    ddl_barrier_release_gates::ddl_barrier_release_gates,
    ddl_barrier_summary_components::DdlBarrierSummaryComponents,
    ddl_barrier_summary_types::{
        DdlBarrierReleaseAction, DdlBarrierReleaseBlocker, DdlBarrierReleaseGate,
        DdlBarrierSinkEvidence,
    },
    normalize_ddl_barrier, normalize_ddl_barrier_ack, DdlBarrier, DdlBarrierAck, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DdlBarrierSummary {
    pub source_id: String,
    #[serde(default)]
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub barrier_lsn: String,
    pub schema_version: String,
    pub cdc_transaction_boundary: String,
    #[serde(default)]
    pub propagation_boundary: Option<String>,
    #[serde(default)]
    pub propagation_decisions: Vec<String>,
    #[serde(default)]
    pub propagation_policy_sha256: Option<String>,
    pub required_sink_count: usize,
    pub acked_sink_count: usize,
    pub pending_sink_count: usize,
    pub rejected_sink_count: usize,
    pub unexpected_ack_count: usize,
    pub acked_sinks: Vec<String>,
    pub pending_sinks: Vec<String>,
    pub rejected_sinks: Vec<String>,
    pub unexpected_sinks: Vec<String>,
    pub sink_evidence: Vec<DdlBarrierSinkEvidence>,
    pub release_blockers: Vec<String>,
    #[serde(default)]
    pub release_blocker_codes: Vec<String>,
    #[serde(default)]
    pub release_blocker_details: Vec<DdlBarrierReleaseBlocker>,
    #[serde(default)]
    pub release_actions: Vec<DdlBarrierReleaseAction>,
    pub release_gates: Vec<DdlBarrierReleaseGate>,
    pub release_dml: bool,
    pub requires_global_partition_pause: bool,
}

impl DdlBarrierSummary {
    pub fn try_from_barrier_and_acks(
        barrier: DdlBarrier,
        acks: Vec<DdlBarrierAck>,
    ) -> Result<Self> {
        let barrier = normalize_ddl_barrier(barrier)?;
        let acks = acks
            .into_iter()
            .map(normalize_ddl_barrier_ack)
            .collect::<Result<Vec<_>>>()?;
        validate_ack_boundaries(&barrier, &acks)?;
        Ok(Self::from_validated_barrier_and_acks(barrier, acks))
    }

    #[cfg(test)]
    pub(crate) fn from_barrier_and_acks(barrier: DdlBarrier, acks: Vec<DdlBarrierAck>) -> Self {
        Self::from_validated_barrier_and_acks(barrier, acks)
    }

    fn from_validated_barrier_and_acks(barrier: DdlBarrier, acks: Vec<DdlBarrierAck>) -> Self {
        let components = DdlBarrierSummaryComponents::from_barrier_and_acks(&barrier, acks);
        let has_required_sinks = !barrier.required_sinks.is_empty();
        let release_dml = has_required_sinks
            && components.pending_sinks.is_empty()
            && components.rejected_sinks.is_empty()
            && components.unexpected_sinks.is_empty();
        let release_blockers = ddl_release_blockers(
            has_required_sinks,
            &components.pending_sinks,
            &components.rejected_sinks,
            &components.unexpected_sinks,
        );
        let release_blocker_codes = ddl_release_blocker_codes(
            has_required_sinks,
            &components.pending_sinks,
            &components.sink_evidence,
            release_dml,
            barrier.requires_global_partition_pause,
        );
        let release_blocker_details = ddl_release_blocker_details(
            DdlBarrierBlockerContext {
                barrier_id: &barrier.barrier_id,
                barrier_lsn: &barrier.barrier_lsn,
                schema_version: &barrier.schema_version,
                has_required_sinks,
                requires_global_partition_pause: barrier.requires_global_partition_pause,
            },
            &components.pending_sinks,
            &components.rejected_sinks,
            &components.unexpected_sinks,
            &components.sink_evidence,
        );
        let release_gates = ddl_barrier_release_gates(
            release_dml,
            barrier.requires_global_partition_pause,
            &release_blockers,
        );
        let release_actions = ddl_barrier_release_actions(
            &barrier.barrier_id,
            &barrier.barrier_lsn,
            &barrier.schema_version,
            &release_blocker_details,
        );

        Self {
            source_id: barrier.source_id,
            database_id: barrier.database_id,
            dataset_id: barrier.dataset_id,
            barrier_id: barrier.barrier_id,
            barrier_lsn: barrier.barrier_lsn,
            schema_version: barrier.schema_version,
            propagation_boundary: propagation_boundary_token(&barrier.cdc_transaction_boundary),
            propagation_decisions: propagation_decision_tokens(&barrier.cdc_transaction_boundary),
            propagation_policy_sha256: propagation_policy_sha256_token(
                &barrier.cdc_transaction_boundary,
            ),
            cdc_transaction_boundary: barrier.cdc_transaction_boundary,
            required_sink_count: barrier.required_sinks.len(),
            acked_sink_count: components.acked_sinks.len(),
            pending_sink_count: components.pending_sinks.len(),
            rejected_sink_count: components.rejected_sinks.len(),
            unexpected_ack_count: components.unexpected_sinks.len(),
            acked_sinks: components.acked_sinks,
            pending_sinks: components.pending_sinks,
            rejected_sinks: components.rejected_sinks,
            unexpected_sinks: components.unexpected_sinks,
            sink_evidence: components.sink_evidence,
            release_blockers,
            release_blocker_codes,
            release_blocker_details,
            release_actions,
            release_gates,
            release_dml,
            requires_global_partition_pause: barrier.requires_global_partition_pause,
        }
    }
}
