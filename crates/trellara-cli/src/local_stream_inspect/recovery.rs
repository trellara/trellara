use trellara_stream_local::{LocalRecoveryInspection, LocalRecoveryTopicEvidence};

use crate::{
    LocalStreamRecoveryActionSummary, LocalStreamRecoveryCursorBlockerSummary,
    LocalStreamRecoverySummary, LocalStreamRecoveryTopicSummary,
};

pub(super) fn local_stream_recovery_summary(
    recovery: LocalRecoveryInspection,
) -> LocalStreamRecoverySummary {
    LocalStreamRecoverySummary {
        recovery_ready: recovery.recovery_ready,
        topic_count: recovery.topic_count,
        total_message_count: recovery.total_message_count,
        replayable_topic_count: recovery.replayable_topic_count,
        torn_tail_topics: recovery
            .torn_tail_topics
            .into_iter()
            .map(local_stream_recovery_topic_summary)
            .collect(),
        recovered_index_topics: recovery
            .recovered_index_topics
            .into_iter()
            .map(local_stream_recovery_topic_summary)
            .collect(),
        unreplayable_topics: recovery.unreplayable_topics,
        cursor_blockers: recovery
            .cursor_blockers
            .into_iter()
            .map(|blocker| LocalStreamRecoveryCursorBlockerSummary {
                group_id: blocker.group_id,
                topic: blocker.topic,
                next_offset: blocker.next_offset,
                status: blocker.status.to_string(),
            })
            .collect(),
        recovery_actions: recovery
            .recovery_actions
            .into_iter()
            .map(|action| LocalStreamRecoveryActionSummary {
                code: action.code.to_string(),
                topic: action.topic,
                command: action.command,
                reason: action.reason,
            })
            .collect(),
    }
}

fn local_stream_recovery_topic_summary(
    topic: LocalRecoveryTopicEvidence,
) -> LocalStreamRecoveryTopicSummary {
    LocalStreamRecoveryTopicSummary {
        topic: topic.topic,
        message_count: topic.message_count,
        last_valid_offset: topic.last_valid_offset,
        valid_bytes: topic.valid_bytes,
        file_bytes: topic.file_bytes,
        torn_tail_bytes: topic.torn_tail_bytes,
        index_status: topic.index_status.to_string(),
    }
}
