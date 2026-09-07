use crate::{ChangeRecord, PartitionChunk, TransactionManifest};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartitionPlanConfig {
    pub partition_count: u32,
    pub key_column: String,
    pub null_key_policy: PartitionNullKeyPolicy,
    pub key_change_policy: PartitionKeyChangePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictChunkPlanConfig {
    pub max_changes_per_chunk: u32,
}

#[derive(Clone, Debug)]
pub struct PartitionPlan {
    pub manifest: TransactionManifest,
    pub chunks: Vec<PartitionChunk>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PartitionNullKeyPolicy {
    Quarantine,
    RouteToDeadLetterPartition,
    RouteToSingletonPartition,
    DeriveFromPrimaryKey,
}

impl std::fmt::Display for PartitionNullKeyPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            PartitionNullKeyPolicy::Quarantine => "quarantine",
            PartitionNullKeyPolicy::RouteToDeadLetterPartition => "route_to_dead_letter_partition",
            PartitionNullKeyPolicy::RouteToSingletonPartition => "route_to_singleton_partition",
            PartitionNullKeyPolicy::DeriveFromPrimaryKey => "derive_from_primary_key",
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PartitionKeyChangePolicy {
    Quarantine,
    EmitMove,
    DualWriteWindow,
    Forbid,
}

impl std::fmt::Display for PartitionKeyChangePolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            PartitionKeyChangePolicy::Quarantine => "quarantine",
            PartitionKeyChangePolicy::EmitMove => "emit_move",
            PartitionKeyChangePolicy::DualWriteWindow => "dual_write_window",
            PartitionKeyChangePolicy::Forbid => "forbid",
        })
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PartitionLocalView {
    pub transaction_id: String,
    pub source_commit_lsn: String,
    pub source_commit_timestamp_ms: i64,
    pub partition_id: u32,
    pub partition_event_count: u32,
    pub global_event_count: u32,
    pub participating_partition_count: u32,
    pub first_total_order: u32,
    pub last_total_order: u32,
    pub transaction_complete: bool,
    pub changes: Vec<ChangeRecord>,
}
