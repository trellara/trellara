use thiserror::Error;

mod checkpoint_rewind;
mod checkpoint_row;
mod checkpoint_validation;
mod ddl_barrier;
mod ddl_barrier_ack_evidence;
mod ddl_barrier_ack_identity;
mod ddl_barrier_ack_rejection;
mod ddl_barrier_ack_validation;
mod ddl_barrier_blocker_details;
mod ddl_barrier_blockers;
mod ddl_barrier_identity;
mod ddl_barrier_policy_evidence;
mod ddl_barrier_policy_validation;
mod ddl_barrier_release_actions;
mod ddl_barrier_release_gates;
mod ddl_barrier_summary;
mod ddl_barrier_summary_components;
mod ddl_barrier_summary_types;
mod ddl_barrier_validation;
mod ddl_barrier_validation_fields;
mod evidence;
mod evidence_row;
mod iceberg_commit;
mod iceberg_commit_validation;
mod in_memory;
mod in_memory_iceberg_commit;
mod in_memory_snapshot;
mod lsn;
mod lsn_validation;
mod partition;
mod partition_checkpoint_row;
mod partition_ddl_ack;
mod partition_ddl_ack_digest;
mod partition_ddl_ack_proof;
mod partition_ddl_ack_validation;
mod partition_ddl_ack_watermark;
mod partition_health;
mod partition_health_actions;
mod partition_health_watermarks;
mod partition_watermark_inputs;
mod postgres;
mod postgres_ddl_barrier;
mod postgres_ddl_barrier_sql;
mod postgres_evidence_reseed;
mod postgres_evidence_snapshot_handoff;
mod postgres_evidence_sql;
mod postgres_evidence_validation;
mod postgres_iceberg_commit;
mod postgres_iceberg_commit_sql;
mod postgres_partition;
mod postgres_partition_sql;
mod postgres_quarantine;
mod postgres_quarantine_sql;
mod postgres_snapshot;
mod postgres_snapshot_sql;
mod postgres_snapshot_table;
mod postgres_sql;
mod quarantine_row;
mod quarantine_validation;
mod raw_cdc_lake_ddl_ack_proof;
mod schema;
mod schema_sql;
mod snapshot;
mod snapshot_handoff_blockers;
mod snapshot_handoff_readiness;
mod snapshot_handoff_readiness_types;
mod snapshot_lookup_validation;
mod snapshot_row;
mod snapshot_run_identity;
mod snapshot_run_validation;
mod snapshot_table_identity;
mod snapshot_table_validation;
mod snapshot_validation;
mod spark_derived_views_ddl_ack_proof;
mod target_postgres_ddl_ack_proof;
mod transaction_key_validation;
mod types;
mod validation;
mod validation_event_digest;
mod validation_event_validation;
mod validation_guards;

pub use ddl_barrier::{
    DdlBarrier, DdlBarrierAck, DdlBarrierStore, DDL_BARRIER_CDC_TRANSACTION_BOUNDARY,
};
pub(crate) use ddl_barrier_ack_validation::normalize_ddl_barrier_ack;
pub use ddl_barrier_identity::DdlBarrierLookup;
pub use ddl_barrier_summary::DdlBarrierSummary;
pub use ddl_barrier_summary_types::{
    DdlBarrierReleaseAction, DdlBarrierReleaseBlocker, DdlBarrierReleaseGate,
    DdlBarrierSinkEvidence,
};
pub(crate) use ddl_barrier_validation::normalize_ddl_barrier;
pub use evidence::{ApplyQuarantine, ReseedEvent, SnapshotHandoffEvent, ValidationEvent};
pub use iceberg_commit::{
    IcebergCommitStore, IcebergTableCommitIntent, IcebergTableCommitKey, IcebergTableCommitReceipt,
    IcebergTableCommitStatus,
};
pub use in_memory::InMemoryCheckpointStore;
pub use lsn::{lsn_shape_is_valid, parse_lsn};
pub use partition::{PartitionCheckpoint, PartitionWatermarkLag, PartitionWatermarkSummary};
pub use partition_ddl_ack::{
    partition_visibility_ddl_ack_evidence, PartitionVisibilityDdlAckEvidence,
    PartitionVisibilityDdlAckRequest,
};
pub use partition_ddl_ack_digest::partition_watermark_summary_sha256;
pub use partition_ddl_ack_proof::{
    partition_visibility_ddl_ack_detail, partition_visibility_ddl_ack_detail_is_valid,
};
pub use partition_health::{
    PartitionScaleHealthAction, PartitionScaleHealthStatus, PartitionScaleHealthSummary,
};
pub use postgres::PostgresCheckpointStore;
pub use raw_cdc_lake_ddl_ack_proof::{
    raw_cdc_lake_ddl_ack_detail_is_valid, RAW_CDC_LAKE_DDL_RELEASE_GATE,
};
pub use schema::postgres_checkpoint_schema_sql;
pub use snapshot::{SnapshotRun, SnapshotRunState, SnapshotTableProgress};
pub use snapshot_handoff_readiness::{
    snapshot_boundary_copied_rows, snapshot_handoff_readiness_report,
    snapshot_handoff_ready_at_boundary, snapshot_progress_complete_at_boundary,
};
pub use snapshot_handoff_readiness_types::{
    SnapshotHandoffReadinessBlocker, SnapshotHandoffReadinessInput, SnapshotHandoffReadinessReport,
};
pub use spark_derived_views_ddl_ack_proof::{
    spark_derived_views_ddl_ack_detail_is_valid, SPARK_DERIVED_VIEWS_DDL_RELEASE_GATE,
};
pub use target_postgres_ddl_ack_proof::{
    target_postgres_ddl_ack_detail, target_postgres_ddl_ack_detail_is_valid,
    target_postgres_ddl_ack_detail_matches_barrier, target_postgres_ddl_ack_detail_with_boundary,
    target_postgres_ddl_ack_detail_with_digests, TARGET_POSTGRES_DDL_RELEASE_GATE,
};
pub use types::{
    ApplyDecision, CheckpointLag, CheckpointStore, DedupStore, FlowKey, TransactionKey,
};

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("checkpoint store error: {0}")]
    Store(String),
}

pub type Result<T> = std::result::Result<T, CheckpointError>;

#[cfg(test)]
mod tests;
