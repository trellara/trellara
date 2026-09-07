pub const PROTOCOL_VERSION: u32 = 1;

mod checkpoint;
mod ddl_event;
mod ddl_propagation;
mod ddl_propagation_policy;
mod ddl_propagation_summary;
mod ddl_propagation_validation;
mod envelope_ddl_validation;
mod envelope_validation;
mod error;
mod lsn;
mod manifest;
mod manifest_affected_tables;
mod manifest_commit_marker;
mod manifest_validation;
mod partition_affected_tables;
mod partition_chunk;
mod partition_contracts;
mod partition_keys;
mod partition_local_view;
mod partition_manifest_plan;
mod partition_operation_order;
mod partition_plan;
mod partition_plan_chunks;
mod partition_plan_guards;
mod partition_rebalance;
mod partition_reconstruct;
mod partition_routing;
mod partition_verify;
mod partition_visibility;
mod row_types;
mod schema_version_validation;
mod transaction_boundary;
mod transaction_boundary_readiness;
mod transaction_envelope;
mod types;

pub use lsn::{format_lsn, parse_lsn};
pub use manifest::*;
pub use manifest_commit_marker::validate_commit_marker;
pub use manifest_validation::validate_transaction_manifest;
pub use partition_chunk::PartitionChunk;
pub use partition_contracts::*;
#[cfg(test)]
pub(crate) use partition_keys::primary_key_bytes_from_row;
pub use partition_local_view::partition_local_view;
pub use partition_plan::{
    partition_local_changes, plan_partitioned_transaction, plan_strict_chunked_transaction,
};
pub use partition_rebalance::{
    plan_partition_rebalance, PartitionLoadObservation, PartitionRebalanceMoveCandidate,
    PartitionRebalancePlan, PartitionRebalancePlanInput, PartitionRebalancePolicy,
    PartitionRebalanceStatus, PARTITION_REBALANCE_VISIBILITY_CONTRACT,
};
pub use partition_reconstruct::{
    reconstruct_barrier_transaction, reconstruct_committed_barrier_transaction,
    validate_manifest_partition_ids,
};
pub use partition_routing::partition_for_key;
pub use partition_verify::validate_partition_chunk;
pub use partition_visibility::{
    barrier_visibility_decision, partition_visibility_decision, BarrierHoldReason,
    BarrierVisibilityDecision, PartitionVisibilityDecision,
};
pub use transaction_boundary::{
    PartitionedScaleDecision, TransactionBoundaryKey, TransactionBoundaryKind,
};
pub use transaction_boundary_readiness::{
    PartitionedScaleManifestEvidence, PartitionedScaleReadiness,
};
pub use transaction_envelope::*;

pub use checkpoint::Checkpoint;
pub use ddl_event::*;
pub use ddl_propagation::*;
pub use ddl_propagation_summary::*;
pub use error::{InvalidChangeFieldError, ProtocolError};
pub use row_types::*;
pub use types::*;

pub fn idempotency_key(
    source_id: &str,
    commit_lsn: &str,
    transaction_id: &str,
    total_order: u32,
) -> String {
    format!("{source_id}:{commit_lsn}:{transaction_id}:{total_order}")
}

#[cfg(test)]
mod tests;
