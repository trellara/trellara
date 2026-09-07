use std::path::Path;

use crate::inspection::inspect_local_stream;
use crate::recovery_actions::recovery_actions;
use crate::{
    LocalCursorStatus, LocalStreamInspection, LocalTopicIndexStatus, LocalTopicInspection, Result,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalRecoveryInspection {
    pub recovery_ready: bool,
    pub topic_count: usize,
    pub total_message_count: i64,
    pub replayable_topic_count: usize,
    pub torn_tail_topics: Vec<LocalRecoveryTopicEvidence>,
    pub recovered_index_topics: Vec<LocalRecoveryTopicEvidence>,
    pub unreplayable_topics: Vec<String>,
    pub cursor_blockers: Vec<LocalRecoveryCursorBlocker>,
    pub recovery_actions: Vec<LocalRecoveryAction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalRecoveryTopicEvidence {
    pub topic: String,
    pub message_count: i64,
    pub last_valid_offset: Option<i64>,
    pub valid_bytes: u64,
    pub file_bytes: u64,
    pub torn_tail_bytes: u64,
    pub index_status: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalRecoveryCursorBlocker {
    pub group_id: String,
    pub topic: String,
    pub next_offset: i64,
    pub status: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalRecoveryAction {
    pub code: &'static str,
    pub topic: Option<String>,
    pub command: String,
    pub reason: String,
}

pub fn inspect_local_recovery(root: impl AsRef<Path>) -> Result<LocalRecoveryInspection> {
    LocalRecoveryInspection::from_stream_inspection(inspect_local_stream(root)?)
}

impl LocalRecoveryInspection {
    pub fn from_stream_inspection(inspection: LocalStreamInspection) -> Result<Self> {
        let topic_count = inspection.topics.len();
        let total_message_count = inspection
            .topics
            .iter()
            .map(|topic| topic.message_count)
            .sum();
        let replayable_topic_count = inspection
            .topics
            .iter()
            .filter(|topic| topic.replayable)
            .count();
        let torn_tail_topics = inspection
            .topics
            .iter()
            .filter(|topic| topic.torn_tail_bytes > 0)
            .map(topic_evidence)
            .collect::<Vec<_>>();
        let recovered_index_topics = inspection
            .topics
            .iter()
            .filter(|topic| topic.index_status != LocalTopicIndexStatus::Healthy)
            .map(topic_evidence)
            .collect::<Vec<_>>();
        let unreplayable_topics = inspection
            .topics
            .iter()
            .filter(|topic| !topic.replayable && topic.message_count > 0)
            .map(|topic| topic.topic.clone())
            .collect::<Vec<_>>();
        let cursor_blockers = inspection
            .cursors
            .into_iter()
            .filter(|cursor| cursor.status != LocalCursorStatus::Ok)
            .map(|cursor| LocalRecoveryCursorBlocker {
                group_id: cursor.group_id,
                topic: cursor.topic,
                next_offset: cursor.next_offset,
                status: cursor.status.as_str(),
            })
            .collect::<Vec<_>>();
        let recovery_actions = recovery_actions(
            &torn_tail_topics,
            &recovered_index_topics,
            &unreplayable_topics,
            &cursor_blockers,
        );

        Ok(Self {
            recovery_ready: torn_tail_topics.is_empty()
                && unreplayable_topics.is_empty()
                && cursor_blockers.is_empty(),
            topic_count,
            total_message_count,
            replayable_topic_count,
            torn_tail_topics,
            recovered_index_topics,
            unreplayable_topics,
            cursor_blockers,
            recovery_actions,
        })
    }
}

fn topic_evidence(topic: &LocalTopicInspection) -> LocalRecoveryTopicEvidence {
    LocalRecoveryTopicEvidence {
        topic: topic.topic.clone(),
        message_count: topic.message_count,
        last_valid_offset: topic.last_valid_offset,
        valid_bytes: topic.valid_bytes,
        file_bytes: topic.file_bytes,
        torn_tail_bytes: topic.torn_tail_bytes,
        index_status: topic.index_status.as_str(),
    }
}
