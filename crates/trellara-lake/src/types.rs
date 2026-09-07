use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeCommitPlan {
    pub source_id: String,
    pub dataset_id: String,
    pub transaction_id: String,
    pub commit_lsn: String,
    pub commit_timestamp_ms: i64,
    pub visibility_boundary: LakeVisibilityBoundary,
    pub operation_count: usize,
    pub operations: Vec<LakeWriteOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LakeVisibilityBoundary {
    StrictEnvelope,
    StrictChunkManifestBarrier {
        global_event_count: u32,
        chunk_count: u32,
    },
    PartitionManifestBarrier {
        global_event_count: u32,
        participating_partition_count: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeWriteOperation {
    pub materialization: LakeMaterialization,
    pub write_kind: LakeWriteKind,
    pub relation: String,
    pub total_order: u32,
    pub record_key: Option<String>,
    pub source_transaction_id: String,
    pub source_commit_lsn: String,
    pub source_commit_timestamp_ms: i64,
    pub row: Vec<LakeColumnValue>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LakeMaterialization {
    RawCdc,
    CurrentState,
    Scd2History,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LakeWriteKind {
    AppendEvent,
    UpsertCurrent,
    DeleteCurrent,
    TruncateCurrent,
    InsertVersion,
    CloseVersion,
    TruncateHistory,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeColumnValue {
    pub name: String,
    pub value: LakeValue,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum LakeValue {
    Null,
    Text(String),
    BinaryByteCount(usize),
    UnchangedToast,
}
