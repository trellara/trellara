use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{RelationId, RowImage};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, prost::Enumeration)]
#[repr(i32)]
pub enum Operation {
    Unspecified = 0,
    Insert = 1,
    Update = 2,
    Delete = 3,
    Truncate = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, prost::Enumeration)]
#[repr(i32)]
pub enum ReplicaIdentity {
    Unspecified = 0,
    Default = 1,
    Index = 2,
    Full = 3,
    Nothing = 4,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct ChangeRecord {
    #[prost(string, tag = "1")]
    pub transaction_id: String,
    #[prost(uint32, tag = "2")]
    pub total_order: u32,
    #[prost(uint32, tag = "3")]
    pub table_order: u32,
    #[prost(uint32, tag = "4")]
    pub partition_order: u32,
    #[prost(message, optional, tag = "5")]
    pub relation: Option<RelationId>,
    #[prost(enumeration = "Operation", tag = "6")]
    pub operation: i32,
    #[prost(enumeration = "ReplicaIdentity", tag = "7")]
    pub replica_identity: i32,
    #[prost(message, optional, tag = "8")]
    pub before: Option<RowImage>,
    #[prost(message, optional, tag = "9")]
    pub after: Option<RowImage>,
    #[prost(string, tag = "10")]
    pub idempotency_key: String,
}
