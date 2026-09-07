use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalStreamInspection {
    pub root: PathBuf,
    pub topics: Vec<LocalTopicInspection>,
    pub cursors: Vec<LocalCursorInspection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalTopicInspection {
    pub topic: String,
    pub message_count: i64,
    pub last_valid_offset: Option<i64>,
    pub last_proof_headers: Vec<LocalTopicProofHeader>,
    pub replayable: bool,
    pub valid_bytes: u64,
    pub file_bytes: u64,
    pub torn_tail_bytes: u64,
    pub index_entries: i64,
    pub index_bytes: u64,
    pub index_status: LocalTopicIndexStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalTopicProofHeader {
    pub key: String,
    pub value: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LocalTopicIndexStatus {
    Healthy,
    MissingRebuilt,
    StaleRebuilt,
    CorruptRebuilt,
}

impl LocalTopicIndexStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::MissingRebuilt => "missing_rebuilt",
            Self::StaleRebuilt => "stale_rebuilt",
            Self::CorruptRebuilt => "corrupt_rebuilt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCursorInspection {
    pub group_id: String,
    pub topic: String,
    pub next_offset: i64,
    pub topic_message_count: Option<i64>,
    pub pending_messages: Option<i64>,
    pub status: LocalCursorStatus,
}

impl LocalCursorInspection {
    pub fn from_topic_depth(
        group_id: impl Into<String>,
        topic: impl Into<String>,
        next_offset: i64,
        topic_message_count: Option<i64>,
    ) -> Self {
        let status = match topic_message_count {
            Some(message_count) if next_offset > message_count => LocalCursorStatus::AheadOfTopic,
            Some(_) => LocalCursorStatus::Ok,
            None => LocalCursorStatus::MissingTopic,
        };
        Self {
            group_id: group_id.into(),
            topic: topic.into(),
            next_offset,
            topic_message_count,
            pending_messages: topic_message_count
                .map(|message_count| message_count.saturating_sub(next_offset).max(0)),
            status,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LocalCursorStatus {
    Ok,
    AheadOfTopic,
    MissingTopic,
}

impl LocalCursorStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::AheadOfTopic => "ahead_of_topic",
            Self::MissingTopic => "missing_topic",
        }
    }
}
