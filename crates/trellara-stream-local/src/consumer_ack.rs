use trellara_stream::StreamMessage;

use crate::cursor_storage::{read_cursor, write_cursor};
use crate::index::recover_topic_index;
use crate::paths::{topic_index_path, topic_path};
use crate::{LocalConsumerConfig, LocalStreamError, Result};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocalAckOutcome {
    pub previous_next_offset: i64,
    pub acked_next_offset: i64,
    pub durable_next_offset: i64,
    pub advanced: bool,
}

pub(crate) fn ack_local(
    config: &LocalConsumerConfig,
    message: &StreamMessage,
) -> Result<LocalAckOutcome> {
    let position = message
        .position
        .as_ref()
        .ok_or(LocalStreamError::MissingPosition)?;
    if !config.topics.iter().any(|topic| topic == &position.topic) {
        return Err(LocalStreamError::UnknownAckTopic {
            topic: position.topic.clone(),
            topics: config.topics.clone(),
        });
    }
    if position.partition != 0 {
        return Err(LocalStreamError::UnsupportedAckPartition {
            topic: position.topic.clone(),
            partition: position.partition,
        });
    }
    if position.offset < 0 {
        return Err(LocalStreamError::NegativeCursorOffset {
            offset: position.offset,
        });
    }
    let acked_next_offset =
        position
            .offset
            .checked_add(1)
            .ok_or(LocalStreamError::CursorOffsetOverflow {
                offset: position.offset,
            })?;
    let topic_message_count = durable_topic_message_count(config, &position.topic)?;
    if position.offset >= topic_message_count {
        return Err(LocalStreamError::CursorOffsetAhead {
            topic: position.topic.clone(),
            offset: position.offset,
            message_count: topic_message_count,
        });
    }
    let current_next_offset = read_cursor(config, &position.topic)?;
    if position.offset > current_next_offset {
        return Err(LocalStreamError::NonContiguousAck {
            topic: position.topic.clone(),
            offset: position.offset,
            current_next_offset,
        });
    }
    let durable_next_offset = if position.offset == current_next_offset {
        acked_next_offset
    } else {
        current_next_offset
    };
    write_cursor(config, &position.topic, durable_next_offset)?;
    Ok(LocalAckOutcome {
        previous_next_offset: current_next_offset,
        acked_next_offset,
        durable_next_offset,
        advanced: durable_next_offset > current_next_offset,
    })
}

fn durable_topic_message_count(config: &LocalConsumerConfig, topic: &str) -> Result<i64> {
    let path = topic_path(&config.root, topic);
    if !path.exists() {
        return Ok(0);
    }
    let state = recover_topic_index(
        &path,
        &topic_index_path(&config.root, topic),
        config.durability,
    )?;
    i64::try_from(state.positions.len()).map_err(|source| LocalStreamError::Io {
        path: path.display().to_string(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
    })
}
