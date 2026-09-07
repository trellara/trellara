use crate::{
    LocalBarrierTopicInspectSummary, LocalCursorInspectSummary, LocalStreamHealthSummary,
    LocalTopicInspectSummary,
};

impl LocalStreamHealthSummary {
    pub(crate) fn from_details(
        barrier_topics: &LocalBarrierTopicInspectSummary,
        topics: &[LocalTopicInspectSummary],
        cursors: &[LocalCursorInspectSummary],
    ) -> Self {
        let total_messages = topics.iter().map(|topic| topic.message_count).sum();
        let total_pending_messages = cursors
            .iter()
            .filter_map(|cursor| cursor.pending_messages)
            .sum();
        let total_valid_bytes = topics.iter().map(|topic| topic.valid_bytes).sum();
        let total_file_bytes = topics.iter().map(|topic| topic.file_bytes).sum();
        let torn_tail_bytes = topics.iter().map(|topic| topic.torn_tail_bytes).sum();
        let rebuilt_index_topics = topics
            .iter()
            .filter(|topic| topic.index_status != "healthy")
            .count();
        let unhealthy_cursors = cursors
            .iter()
            .filter(|cursor| cursor.cursor_status != "ok")
            .count();
        let missing_configured_topics = barrier_topics.missing_topics.len();
        let status = if missing_configured_topics > 0 || unhealthy_cursors > 0 {
            "degraded"
        } else if torn_tail_bytes > 0 || rebuilt_index_topics > 0 {
            "recovered"
        } else {
            "clean"
        };

        Self {
            status: status.to_string(),
            topic_count: topics.len(),
            cursor_count: cursors.len(),
            total_messages,
            total_pending_messages,
            total_valid_bytes,
            total_file_bytes,
            torn_tail_bytes,
            rebuilt_index_topics,
            unhealthy_cursors,
            missing_configured_topics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn barrier_topics(missing_topics: Vec<String>) -> LocalBarrierTopicInspectSummary {
        LocalBarrierTopicInspectSummary {
            required: !missing_topics.is_empty(),
            expected_topic_count: missing_topics.len(),
            observed_configured_topic_count: 0,
            missing_topics,
            manifest_topic_present: None,
            commit_topic_present: None,
        }
    }

    fn topic(index_status: &str, torn_tail_bytes: u64) -> LocalTopicInspectSummary {
        LocalTopicInspectSummary {
            topic: "trellara.local-source.retail-sales.strict".to_string(),
            message_count: 3,
            last_valid_offset: Some(2),
            last_proof_headers: Vec::new(),
            replayable: true,
            valid_bytes: 90,
            file_bytes: 90 + torn_tail_bytes,
            torn_tail_bytes,
            index_entries: 3,
            index_bytes: 24,
            index_status: index_status.to_string(),
            transaction_locate_command: None,
            replay_last_valid_command: None,
        }
    }

    fn cursor(cursor_status: &str) -> LocalCursorInspectSummary {
        LocalCursorInspectSummary {
            group_id: "applier".to_string(),
            topic: "trellara.local-source.retail-sales.strict".to_string(),
            next_offset: 1,
            topic_message_count: Some(3),
            pending_messages: Some(2),
            cursor_status: cursor_status.to_string(),
        }
    }

    #[test]
    fn degraded_health_takes_precedence_over_recovered_artifacts() {
        let summary = LocalStreamHealthSummary::from_details(
            &barrier_topics(vec!["trellara.local-source.retail-sales.commit".to_string()]),
            &[topic("stale_rebuilt", 4)],
            &[cursor("ahead_of_topic")],
        );

        assert_eq!(summary.status, "degraded");
        assert_eq!(summary.missing_configured_topics, 1);
        assert_eq!(summary.unhealthy_cursors, 1);
        assert_eq!(summary.rebuilt_index_topics, 1);
        assert_eq!(summary.torn_tail_bytes, 4);
        assert_eq!(summary.total_pending_messages, 2);
    }
}
