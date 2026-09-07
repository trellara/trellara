use crate::recovery_inspection::{
    LocalRecoveryAction, LocalRecoveryCursorBlocker, LocalRecoveryTopicEvidence,
};

pub(crate) fn recovery_actions(
    torn_tail_topics: &[LocalRecoveryTopicEvidence],
    recovered_index_topics: &[LocalRecoveryTopicEvidence],
    unreplayable_topics: &[String],
    cursor_blockers: &[LocalRecoveryCursorBlocker],
) -> Vec<LocalRecoveryAction> {
    let mut actions = Vec::new();
    actions.extend(torn_tail_topics.iter().map(torn_tail_action));
    actions.extend(recovered_index_topics.iter().map(rebuilt_index_action));
    actions.extend(unreplayable_topics.iter().map(unreplayable_topic_action));
    actions.extend(cursor_blockers.iter().map(cursor_blocker_action));
    actions
}

fn torn_tail_action(topic: &LocalRecoveryTopicEvidence) -> LocalRecoveryAction {
    LocalRecoveryAction {
        code: "truncate_torn_tail_on_next_publish",
        topic: Some(topic.topic.clone()),
        command: "trellara relay --config <flow>".to_string(),
        reason: format!(
            "{} has {} torn tail bytes after valid offset {}; next local publish truncates the invalid tail before appending",
            topic.topic,
            topic.torn_tail_bytes,
            topic
                .last_valid_offset
                .map_or_else(|| "none".to_string(), |offset| offset.to_string())
        ),
    }
}

fn rebuilt_index_action(topic: &LocalRecoveryTopicEvidence) -> LocalRecoveryAction {
    LocalRecoveryAction {
        code: "index_rebuilt_from_segment_log",
        topic: Some(topic.topic.clone()),
        command: format!(
            "trellara stream inspect-local --config <flow> --topic {}",
            topic.topic
        ),
        reason: format!(
            "{} index status is {}; recovered offsets are replayable through valid byte {}",
            topic.topic, topic.index_status, topic.valid_bytes
        ),
    }
}

fn unreplayable_topic_action(topic: &String) -> LocalRecoveryAction {
    LocalRecoveryAction {
        code: "investigate_unreplayable_topic",
        topic: Some(topic.clone()),
        command: format!("trellara stream inspect-local --config <flow> --topic {topic}"),
        reason: format!("{topic} contains messages that cannot be replayed safely"),
    }
}

fn cursor_blocker_action(blocker: &LocalRecoveryCursorBlocker) -> LocalRecoveryAction {
    let command = match blocker.status {
        "ahead_of_topic" => format!(
            "trellara stream seek-local --config <flow> --consumer-group {} --topic {} --next-offset <last-valid-offset>",
            blocker.group_id, blocker.topic
        ),
        "missing_topic" => "trellara stream inspect-local --config <flow>".to_string(),
        _ => "trellara stream inspect-local --config <flow>".to_string(),
    };
    LocalRecoveryAction {
        code: "repair_cursor_blocker",
        topic: Some(blocker.topic.clone()),
        command,
        reason: format!(
            "consumer group {} has cursor {} at offset {} for {}",
            blocker.group_id, blocker.status, blocker.next_offset, blocker.topic
        ),
    }
}
