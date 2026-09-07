use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamInspectSummary {
    pub(crate) root: String,
    pub(crate) configured_topics: Vec<String>,
    pub(crate) health: LocalStreamHealthSummary,
    pub(crate) recovery: LocalStreamRecoverySummary,
    pub(crate) barrier_topics: LocalBarrierTopicInspectSummary,
    pub(crate) topics: Vec<LocalTopicInspectSummary>,
    pub(crate) cursors: Vec<LocalCursorInspectSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamHealthSummary {
    pub(crate) status: String,
    pub(crate) topic_count: usize,
    pub(crate) cursor_count: usize,
    pub(crate) total_messages: i64,
    pub(crate) total_pending_messages: i64,
    pub(crate) total_valid_bytes: u64,
    pub(crate) total_file_bytes: u64,
    pub(crate) torn_tail_bytes: u64,
    pub(crate) rebuilt_index_topics: usize,
    pub(crate) unhealthy_cursors: usize,
    pub(crate) missing_configured_topics: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamRecoverySummary {
    pub(crate) recovery_ready: bool,
    pub(crate) topic_count: usize,
    pub(crate) total_message_count: i64,
    pub(crate) replayable_topic_count: usize,
    pub(crate) torn_tail_topics: Vec<LocalStreamRecoveryTopicSummary>,
    pub(crate) recovered_index_topics: Vec<LocalStreamRecoveryTopicSummary>,
    pub(crate) unreplayable_topics: Vec<String>,
    pub(crate) cursor_blockers: Vec<LocalStreamRecoveryCursorBlockerSummary>,
    pub(crate) recovery_actions: Vec<LocalStreamRecoveryActionSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamRecoveryTopicSummary {
    pub(crate) topic: String,
    pub(crate) message_count: i64,
    pub(crate) last_valid_offset: Option<i64>,
    pub(crate) valid_bytes: u64,
    pub(crate) file_bytes: u64,
    pub(crate) torn_tail_bytes: u64,
    pub(crate) index_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamRecoveryCursorBlockerSummary {
    pub(crate) group_id: String,
    pub(crate) topic: String,
    pub(crate) next_offset: i64,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamRecoveryActionSummary {
    pub(crate) code: String,
    pub(crate) topic: Option<String>,
    pub(crate) command: String,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalBarrierTopicInspectSummary {
    pub(crate) required: bool,
    pub(crate) expected_topic_count: usize,
    pub(crate) observed_configured_topic_count: usize,
    pub(crate) missing_topics: Vec<String>,
    pub(crate) manifest_topic_present: Option<bool>,
    pub(crate) commit_topic_present: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalTopicInspectSummary {
    pub(crate) topic: String,
    pub(crate) message_count: i64,
    pub(crate) last_valid_offset: Option<i64>,
    pub(crate) last_proof_headers: Vec<LocalTopicProofHeaderSummary>,
    pub(crate) replayable: bool,
    pub(crate) valid_bytes: u64,
    pub(crate) file_bytes: u64,
    pub(crate) torn_tail_bytes: u64,
    pub(crate) index_entries: i64,
    pub(crate) index_bytes: u64,
    pub(crate) index_status: String,
    pub(crate) transaction_locate_command: Option<String>,
    pub(crate) replay_last_valid_command: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalTopicProofHeaderSummary {
    pub(crate) key: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalCursorInspectSummary {
    pub(crate) group_id: String,
    pub(crate) topic: String,
    pub(crate) next_offset: i64,
    pub(crate) topic_message_count: Option<i64>,
    pub(crate) pending_messages: Option<i64>,
    pub(crate) cursor_status: String,
}
