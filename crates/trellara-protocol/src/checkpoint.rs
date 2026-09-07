use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct Checkpoint {
    #[prost(string, tag = "1")]
    pub source_id: String,
    #[prost(string, tag = "2")]
    pub dataset_id: String,
    #[prost(string, tag = "3")]
    pub last_seen_lsn: String,
    #[prost(string, tag = "4")]
    pub last_durable_lsn: String,
    #[prost(string, tag = "5")]
    pub last_applied_lsn: String,
}
