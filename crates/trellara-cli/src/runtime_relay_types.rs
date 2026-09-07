use serde::Serialize;
use trellara_stream::{PublishAck, StreamMessage};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct RelaySummary {
    pub(crate) published_transactions: u64,
    pub(crate) published_messages: u64,
    pub(crate) last_commit_lsn: Option<String>,
    pub(crate) last_topic: Option<String>,
    pub(crate) last_partition: Option<i32>,
    pub(crate) last_offset: Option<i64>,
    pub(crate) source_ack_contract: Option<String>,
    pub(crate) source_ack_lsn: Option<String>,
    pub(crate) source_ack_after_durable_publish: bool,
    pub(crate) source_ack_publish_destinations_match: bool,
    pub(crate) source_ack_publish_destination_count: usize,
    #[serde(skip)]
    pub(crate) latest_publish_messages: Vec<StreamMessage>,
    #[serde(skip)]
    pub(crate) latest_publish_acks: Vec<PublishAck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) local_stream_evidence: Option<LocalRunStreamEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalRunStreamEvidence {
    pub(crate) status: String,
    pub(crate) total_messages: i64,
    pub(crate) total_pending_messages: i64,
    pub(crate) torn_tail_bytes: u64,
    pub(crate) rebuilt_index_topics: usize,
    pub(crate) unhealthy_cursors: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_ack_durability_proof: Option<LocalSourceAckEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) last_publish_ack_proof: Option<LocalPublishAckEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalSourceAckEvidence {
    pub(crate) contract: String,
    pub(crate) durability: String,
    pub(crate) crash_safe_ack: bool,
    pub(crate) expected_publish_messages: usize,
    pub(crate) durable_publish_acks: usize,
    pub(crate) all_publish_acks_proven: bool,
    pub(crate) proofed_ack_count: usize,
    pub(crate) publish_ack_proofs: Vec<LocalPublishAckEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalPublishAckEvidence {
    pub(crate) contract: String,
    pub(crate) durability: String,
    pub(crate) crash_safe_ack: bool,
    pub(crate) topic: String,
    pub(crate) partition: i32,
    pub(crate) offset: i64,
    pub(crate) key: String,
    pub(crate) indexed: bool,
    pub(crate) replayable: bool,
    pub(crate) index_status: String,
    pub(crate) torn_tail_bytes: u64,
}
