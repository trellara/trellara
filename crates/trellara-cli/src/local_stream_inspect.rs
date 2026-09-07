#[cfg(feature = "local-stream")]
use std::collections::BTreeSet;

#[cfg(feature = "local-stream")]
use trellara_stream_local::{
    inspect_local_stream, LocalRecoveryInspection, LocalStreamInspection, LocalTopicInspection,
};

#[cfg(feature = "local-stream")]
use crate::{
    CliError, LocalBarrierTopicInspectSummary, LocalCursorInspectSummary, LocalStreamHealthSummary,
    LocalStreamInspectSummary, LocalTopicInspectSummary, Result, StreamConfig, TrellaraConfig,
};
#[cfg(not(feature = "local-stream"))]
use crate::{LocalStreamInspectSummary, Result, TrellaraConfig};

#[cfg(feature = "local-stream")]
#[path = "local_stream_inspect/recovery.rs"]
mod recovery;

#[cfg(feature = "local-stream")]
pub(crate) fn inspect_configured_local_stream(
    config: &TrellaraConfig,
) -> Result<LocalStreamInspectSummary> {
    let (path, _) = match &config.stream {
        StreamConfig::Local {
            path,
            consumer_group,
            ..
        } => (path, consumer_group),
        StreamConfig::Kafka { .. } => {
            return Err(CliError::InvalidConfig(
                "stream.kind must be local for local stream inspection".to_string(),
            ));
        }
    };
    LocalStreamInspectSummary::from_inspection(
        inspect_local_stream(path)?,
        config.local_stream_topics()?,
    )
}

#[cfg(not(feature = "local-stream"))]
pub(crate) fn inspect_configured_local_stream(
    _config: &TrellaraConfig,
) -> Result<LocalStreamInspectSummary> {
    Err(crate::local_stream_feature_disabled())
}

#[cfg(feature = "local-stream")]
impl LocalStreamInspectSummary {
    pub(crate) fn from_inspection(
        inspection: LocalStreamInspection,
        configured_topics: Vec<String>,
    ) -> Result<Self> {
        let recovery = recovery::local_stream_recovery_summary(
            LocalRecoveryInspection::from_stream_inspection(inspection.clone())?,
        );
        let observed_topics = inspection
            .topics
            .iter()
            .map(|topic| topic.topic.clone())
            .collect::<BTreeSet<_>>();
        let barrier_topics =
            LocalBarrierTopicInspectSummary::from_topics(&configured_topics, &observed_topics);
        let topics = inspection
            .topics
            .into_iter()
            .map(LocalTopicInspectSummary::from_topic)
            .collect::<Vec<_>>();
        let cursors = inspection
            .cursors
            .into_iter()
            .map(LocalCursorInspectSummary::from_cursor)
            .collect::<Vec<_>>();
        let health = LocalStreamHealthSummary::from_details(&barrier_topics, &topics, &cursors);

        Ok(Self {
            root: inspection.root.display().to_string(),
            configured_topics,
            health,
            recovery,
            barrier_topics,
            topics,
            cursors,
        })
    }
}

#[cfg(feature = "local-stream")]
impl LocalTopicInspectSummary {
    fn from_topic(topic: LocalTopicInspection) -> Self {
        let transaction_locate_command = topic.replayable.then(|| {
            format!(
                "trellara stream locate-local --config <config> --transaction-id <tx> --commit-lsn <lsn> --topic {}",
                topic.topic
            )
        });
        let replay_last_valid_command = topic.last_valid_offset.map(|offset| {
            format!(
                "trellara stream seek-local --config <config> --topic {} --next-offset {}",
                topic.topic, offset
            )
        });
        Self {
            topic: topic.topic,
            message_count: topic.message_count,
            last_valid_offset: topic.last_valid_offset,
            last_proof_headers: topic
                .last_proof_headers
                .into_iter()
                .map(|header| crate::LocalTopicProofHeaderSummary {
                    key: header.key,
                    value: header.value,
                })
                .collect(),
            replayable: topic.replayable,
            valid_bytes: topic.valid_bytes,
            file_bytes: topic.file_bytes,
            torn_tail_bytes: topic.torn_tail_bytes,
            index_entries: topic.index_entries,
            index_bytes: topic.index_bytes,
            index_status: topic.index_status.as_str().to_string(),
            transaction_locate_command,
            replay_last_valid_command,
        }
    }
}

#[cfg(feature = "local-stream")]
impl LocalCursorInspectSummary {
    fn from_cursor(cursor: trellara_stream_local::LocalCursorInspection) -> Self {
        Self {
            group_id: cursor.group_id,
            topic: cursor.topic,
            next_offset: cursor.next_offset,
            topic_message_count: cursor.topic_message_count,
            pending_messages: cursor.pending_messages,
            cursor_status: cursor.status.as_str().to_string(),
        }
    }
}
