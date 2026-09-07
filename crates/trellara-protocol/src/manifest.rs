use prost::Message;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::xxh3_64;

use crate::RelationId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, prost::Enumeration)]
#[repr(i32)]
pub enum ManifestBoundaryMode {
    Unspecified = 0,
    StrictChunkedTransactionOrder = 1,
    PartitionedScale = 2,
}

impl ManifestBoundaryMode {
    pub fn status_mode(self) -> &'static str {
        match self {
            Self::StrictChunkedTransactionOrder => "strict_chunked_transaction_order",
            Self::PartitionedScale => "partitioned_scale_mode",
            Self::Unspecified => "manifest_barrier_transaction",
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct ManifestPartition {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(uint32, tag = "2")]
    pub event_count: u32,
    #[prost(uint32, tag = "3")]
    pub first_total_order: u32,
    #[prost(uint32, tag = "4")]
    pub last_total_order: u32,
    #[prost(uint64, tag = "5")]
    pub checksum: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct AffectedTable {
    #[prost(message, optional, tag = "1")]
    pub relation: Option<RelationId>,
    #[prost(uint32, tag = "2")]
    pub event_count: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct TransactionManifest {
    #[prost(string, tag = "1")]
    pub transaction_id: String,
    #[prost(string, tag = "2")]
    pub source_commit_lsn: String,
    #[prost(int64, tag = "3")]
    pub source_commit_timestamp_ms: i64,
    #[prost(uint32, tag = "4")]
    pub global_event_count: u32,
    #[prost(message, repeated, tag = "5")]
    pub partitions: Vec<ManifestPartition>,
    #[prost(message, repeated, tag = "6")]
    pub affected_tables: Vec<AffectedTable>,
    #[prost(enumeration = "ManifestBoundaryMode", tag = "7")]
    pub boundary_mode: i32,
}

impl TransactionManifest {
    pub fn compute_checksum(&self) -> u64 {
        xxh3_64(&self.encode_to_vec())
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct TransactionCommitMarker {
    #[prost(string, tag = "1")]
    pub transaction_id: String,
    #[prost(string, tag = "2")]
    pub source_commit_lsn: String,
    #[prost(int64, tag = "3")]
    pub source_commit_timestamp_ms: i64,
    #[prost(uint32, tag = "4")]
    pub global_event_count: u32,
    #[prost(uint32, tag = "5")]
    pub participating_partition_count: u32,
    #[prost(uint64, tag = "6")]
    pub manifest_checksum: u64,
}
