use std::collections::BTreeSet;

use crate::LocalBarrierTopicInspectSummary;

impl LocalBarrierTopicInspectSummary {
    pub(crate) fn from_topics(
        configured_topics: &[String],
        observed_topics: &BTreeSet<String>,
    ) -> Self {
        let required = configured_topics
            .iter()
            .any(|topic| is_manifest_topic(topic) || is_commit_topic(topic));
        let observed_configured_topic_count = configured_topics
            .iter()
            .filter(|topic| observed_topics.contains(*topic))
            .count();
        let missing_topics = configured_topics
            .iter()
            .filter(|topic| !observed_topics.contains(*topic))
            .cloned()
            .collect::<Vec<_>>();
        let manifest_topic_present = configured_topics
            .iter()
            .find(|topic| is_manifest_topic(topic))
            .map(|topic| observed_topics.contains(topic));
        let commit_topic_present = configured_topics
            .iter()
            .find(|topic| is_commit_topic(topic))
            .map(|topic| observed_topics.contains(topic));

        Self {
            required,
            expected_topic_count: configured_topics.len(),
            observed_configured_topic_count,
            missing_topics,
            manifest_topic_present,
            commit_topic_present,
        }
    }
}

fn is_manifest_topic(topic: &str) -> bool {
    topic.rsplit('.').next() == Some("manifest")
}

fn is_commit_topic(topic: &str) -> bool {
    topic.rsplit('.').next() == Some("commit")
}
