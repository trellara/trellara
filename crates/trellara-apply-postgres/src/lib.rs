mod applier;
mod applier_ddl;
mod applier_transaction;
mod barrier;
mod barrier_header_clean;
mod barrier_header_context;
mod barrier_header_lookup;
mod barrier_header_payload;
mod barrier_headers;
mod barrier_manifest_evidence_headers;
mod barrier_manifest_evidence_validation;
mod barrier_pending;
mod barrier_pending_stats;
mod barrier_run_stats;
mod barrier_strict_ddl_headers;
mod barrier_strict_headers;
mod checkpoint;
mod checkpoint_evidence;
mod checkpoint_manifest_evidence;
mod checkpoint_quarantine;
mod checkpoint_quarantine_sql;
mod checkpoint_sql;
mod ddl;
mod ddl_ack;
mod ddl_ack_context;
mod ddl_ack_evidence;
mod ddl_ack_validation;
mod ddl_barrier_record;
mod ddl_barrier_requirements;
mod ddl_digest;
mod ddl_dml_replay_proof;
mod ddl_dml_replay_proof_validation;
mod ddl_envelope_apply;
mod ddl_evidence;
mod ddl_plan_digest_validation;
mod ddl_plan_validation;
mod ddl_protocol;
mod ddl_protocol_statement;
mod ddl_release_decision;
mod error;
mod executor;
mod plan;
mod plan_delete;
mod plan_insert;
mod plan_row;
mod plan_sql_assignments;
mod plan_sql_fragments;
mod plan_sql_predicates;
mod plan_truncate;
mod plan_update;
mod sql;
mod types;
mod worker;
mod worker_ack_messages;
mod worker_buffer;
mod worker_buffer_chunk;
mod worker_buffer_commit;
mod worker_buffer_manifest;
mod worker_error;
mod worker_ready_envelope;
mod worker_stats_counter;
mod worker_strict;
mod worker_transaction;

#[cfg(test)]
use checkpoint_evidence::validate_apply_checkpoint_evidence;
#[cfg(test)]
use checkpoint_quarantine::quarantine_reason;
#[cfg(test)]
use executor::validate_affected_rows;

pub use applier::PostgresApplier;
pub(crate) use applier_transaction::apply_envelope_transactionally;
pub use barrier::{ApplyStep, BarrierApplyStep};
pub use barrier_pending_stats::BarrierPendingStats;
pub use barrier_run_stats::ApplyRunStats;
pub(crate) use ddl::TARGET_DDL_TRANSACTION_BOUNDARY;
pub use ddl::{
    execute_target_ddl_transaction, plan_target_ddl_transaction, TargetDdlApplyOutcome,
    TargetDdlApplyPlan, TargetDdlStatement, TargetDdlTransactionPlan,
};
pub use ddl_ack_context::TargetDdlAckContext;
pub use ddl_ack_evidence::TargetDdlAckEvidence;
pub use ddl_barrier_record::record_target_ddl_barrier_from_envelope;
pub use ddl_barrier_requirements::TargetDdlBarrierRequirements;
pub(crate) use ddl_dml_replay_proof::target_ddl_dml_replay_proof;
pub use ddl_dml_replay_proof::{TargetDdlDmlReplayProof, TARGET_DDL_DML_REPLAY_PROOF_CONTRACT};
pub use ddl_envelope_apply::{
    apply_target_ddl_envelope_and_record_ack, apply_target_ddl_envelope_then_dml,
    TargetDdlEnvelopeAndDmlApplyOutcome, TargetDdlEnvelopeApplyOutcome,
};
pub use ddl_evidence::{
    target_ddl_apply_plan_from_evidence, TargetDdlApplyPlanEvidence, TargetDdlStatementEvidence,
};
pub(crate) use ddl_plan_validation::{
    validate_ddl_apply_plan_header, validate_ddl_transaction_plan,
};
pub use ddl_protocol::{
    target_ddl_apply_plan_from_envelope, target_ddl_barrier_from_envelope,
    target_ddl_barrier_from_envelope_with_requirements,
};
pub(crate) use ddl_protocol_statement::{collect_statement, ordered_ddl_events};
pub use ddl_release_decision::{target_ddl_release_decision, TargetDmlReleaseDecision};
pub use error::{ApplyError, Result};
pub(crate) use plan::plan_envelope_with_policies;
pub use plan::{plan_change, plan_envelope};
pub use sql::{SqlStatement, SqlValue, SqlValueData};
pub use types::{
    ApplyDecision, ApplyOutcome, ApplyTablePolicy, EnvelopeApplier, PostgresApplyConfig,
};
pub use worker::BarrierAwareApplyWorker;
pub use worker_error::{ApplyWorkerError, ApplyWorkerResult};
pub(crate) use worker_stats_counter::checked_worker_stat_add;
pub use worker_strict::ApplyWorker;

#[cfg(test)]
mod tests;
