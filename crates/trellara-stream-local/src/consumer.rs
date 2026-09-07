use std::fs;

use trellara_stream::{StreamMessage, StreamPosition};

use crate::cursor_storage::read_cursor;
use crate::index_read::read_record_at;
use crate::paths::{cursor_dir, topic_index_path, topic_path};
use crate::{
    ack_local, LocalAckOutcome, LocalConsumer, LocalConsumerConfig, LocalReadCursor,
    LocalStreamError, Result,
};

impl LocalConsumer {
    pub fn new(config: LocalConsumerConfig) -> Result<Self> {
        config.validate()?;
        fs::create_dir_all(cursor_dir(&config.root, &config.group_id)).map_err(|source| {
            LocalStreamError::Io {
                path: cursor_dir(&config.root, &config.group_id)
                    .display()
                    .to_string(),
                source,
            }
        })?;
        Ok(Self {
            config,
            read_cursors: Default::default(),
        })
    }

    pub fn ack_with_outcome(&mut self, message: &StreamMessage) -> Result<LocalAckOutcome> {
        let outcome = ack_local(&self.config, message)?;
        if let Some(position) = &message.position {
            self.read_cursors
                .entry(position.topic.clone())
                .and_modify(|read_cursor| {
                    read_cursor.durable_offset = outcome.durable_next_offset;
                    read_cursor.next_offset =
                        read_cursor.next_offset.max(outcome.durable_next_offset);
                })
                .or_insert(LocalReadCursor {
                    durable_offset: outcome.durable_next_offset,
                    next_offset: outcome.durable_next_offset,
                });
        }
        Ok(outcome)
    }
}

pub(crate) fn next_local(consumer: &mut LocalConsumer) -> Result<Option<StreamMessage>> {
    for topic in &consumer.config.topics {
        let durable_cursor = read_cursor(&consumer.config, topic)?;
        let read_cursor = consumer
            .read_cursors
            .entry(topic.clone())
            .or_insert(LocalReadCursor {
                durable_offset: durable_cursor,
                next_offset: durable_cursor,
            });
        if read_cursor.durable_offset != durable_cursor {
            read_cursor.durable_offset = durable_cursor;
            read_cursor.next_offset = durable_cursor;
        }
        let cursor = read_cursor.next_offset;
        let path = topic_path(&consumer.config.root, topic);
        if !path.exists() {
            continue;
        }
        if let Some(mut message) = read_record_at(
            &path,
            &topic_index_path(&consumer.config.root, topic),
            topic,
            cursor,
            consumer.config.durability,
        )? {
            consumer
                .read_cursors
                .entry(topic.clone())
                .and_modify(|read_cursor| read_cursor.next_offset = cursor.saturating_add(1));
            message.position = Some(StreamPosition {
                topic: topic.clone(),
                partition: 0,
                offset: cursor,
            });
            message.partition = Some(0);
            return Ok(Some(message));
        }
    }
    Ok(None)
}
