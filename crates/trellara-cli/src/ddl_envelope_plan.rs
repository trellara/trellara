use serde::Serialize;
use trellara_apply_postgres::target_ddl_barrier_from_envelope_with_requirements;
use trellara_protocol::{
    ddl_propagation_policy_sha256, summarize_ddl_propagation, TransactionEnvelope,
    DDL_PROPAGATION_CDC_BOUNDARY,
};

use crate::{
    ddl_envelope_barrier_requirements, ddl_envelope_replay_summary, ddl_envelope_steps,
    ddl_event_summary, schema_version_summary, target_ddl_sql_plan,
    transaction_boundary_kind_label, DdlEnvelopeEventSummary, DdlEnvelopeReplaySummary,
    DdlEnvelopeSchemaVersionSummary, Result, TrellaraConfig,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlEnvelopePlanSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) source_boundary_kind: String,
    pub(crate) ddl_event_count: usize,
    pub(crate) dml_change_count: usize,
    pub(crate) barrier_id: Option<String>,
    pub(crate) barrier_schema_version: Option<String>,
    pub(crate) required_sinks: Vec<String>,
    pub(crate) requires_global_partition_pause: bool,
    pub(crate) propagation_boundary: String,
    pub(crate) propagation_decisions: Vec<String>,
    pub(crate) propagation_policy_sha256: String,
    pub(crate) schema_versions: Vec<DdlEnvelopeSchemaVersionSummary>,
    pub(crate) ddl_events: Vec<DdlEnvelopeEventSummary>,
    pub(crate) executable: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) target_sql: Vec<String>,
    pub(crate) dml_replay_after_barrier: Option<DdlEnvelopeReplaySummary>,
    pub(crate) steps: Vec<String>,
}

impl DdlEnvelopePlanSummary {
    pub(crate) fn from_envelope(
        config: &TrellaraConfig,
        envelope: &TransactionEnvelope,
    ) -> Result<Self> {
        let requirements = ddl_envelope_barrier_requirements(config);
        let barrier =
            target_ddl_barrier_from_envelope_with_requirements(envelope, requirements.clone())?;
        let target_plan = target_ddl_sql_plan(envelope, config.target.is_some())?;
        let propagation_summary = summarize_ddl_propagation(&envelope.ddl_events)?;
        let (barrier_id, barrier_schema_version) = barrier
            .map(|barrier| (Some(barrier.barrier_id), Some(barrier.schema_version)))
            .unwrap_or((None, None));
        let dml_replay_after_barrier = ddl_envelope_replay_summary(envelope, barrier_id.as_deref());

        Ok(Self {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            mode: config.dataset.mode.to_string(),
            transaction_id: envelope.transaction_id.clone(),
            commit_lsn: envelope.commit_lsn.clone(),
            source_boundary_kind: transaction_boundary_kind_label(envelope.boundary_kind())
                .to_string(),
            ddl_event_count: envelope.ddl_events.len(),
            dml_change_count: envelope.changes.len(),
            barrier_id,
            barrier_schema_version,
            required_sinks: requirements.required_sinks,
            requires_global_partition_pause: requirements.requires_global_partition_pause,
            propagation_boundary: DDL_PROPAGATION_CDC_BOUNDARY.to_string(),
            propagation_decisions: propagation_summary_tokens(&propagation_summary),
            propagation_policy_sha256: ddl_propagation_policy_sha256(&envelope.ddl_events)?,
            schema_versions: envelope
                .schema_versions
                .iter()
                .map(schema_version_summary)
                .collect(),
            ddl_events: envelope.ddl_events.iter().map(ddl_event_summary).collect(),
            executable: target_plan.executable,
            blockers: target_plan.blockers,
            target_sql: target_plan.target_sql,
            dml_replay_after_barrier,
            steps: ddl_envelope_steps(envelope),
        })
    }
}

fn propagation_summary_tokens(summary: &trellara_protocol::DdlPropagationSummary) -> Vec<String> {
    vec![
        format!("auto_apply:{}", summary.auto_apply),
        format!("manual_review:{}", summary.manual_review),
        format!("unsupported:{}", summary.unsupported),
        format!("target_ack_required:{}", summary.target_ack_required),
    ]
}
