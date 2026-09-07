use serde::{Deserialize, Serialize};

pub const PARTITION_REBALANCE_VISIBILITY_CONTRACT: &str =
    "rebalance is a plan artifact only; runtime ownership movement waits for reviewed manifest, checkpoint, and consumer cutover evidence";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PartitionRebalancePolicy {
    ObserveOnly,
    PlanIfSkewed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PartitionRebalanceStatus {
    IncompleteEvidence,
    Stable,
    Skewed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionLoadObservation {
    pub partition_id: u32,
    pub event_count: u64,
    pub durable_lsn: String,
    pub applied_lsn: String,
    pub blocks_global_applied_watermark: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionRebalancePlanInput {
    pub source_id: String,
    pub dataset_id: String,
    pub expected_partition_count: u32,
    pub policy: PartitionRebalancePolicy,
    pub max_skew_percent: u32,
    pub observations: Vec<PartitionLoadObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionRebalanceMoveCandidate {
    pub from_partition_id: u32,
    pub to_partition_id: u32,
    pub estimated_event_delta: u64,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionRebalancePlan {
    pub source_id: String,
    pub dataset_id: String,
    pub expected_partition_count: u32,
    pub policy: PartitionRebalancePolicy,
    pub status: PartitionRebalanceStatus,
    pub evidence_complete: bool,
    pub runtime_movement_allowed: bool,
    pub visibility_contract: String,
    pub max_skew_percent: u32,
    pub min_event_count: Option<u64>,
    pub max_event_count: Option<u64>,
    pub total_event_count: u64,
    pub skew_ratio_basis_points: Option<u64>,
    pub missing_partitions: Vec<u32>,
    pub blocking_partition_ids: Vec<u32>,
    pub recommended_moves: Vec<PartitionRebalanceMoveCandidate>,
}
