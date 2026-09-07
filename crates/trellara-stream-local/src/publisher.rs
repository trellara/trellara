use std::fs::{self, OpenOptions};

use trellara_stream::{validate_message_shape, PublishAck, StreamError, StreamMessage};

use crate::cursor_storage::sync_dir;
use crate::frame::write_frame;
use crate::index::{checked_offset, recover_topic_index, write_index};
use crate::paths::{topic_dir, topic_index_path, topic_path};
use crate::{validate_topic, LocalPublisher, LocalPublisherConfig, LocalStreamError, Result};

impl LocalPublisher {
    pub fn new(config: LocalPublisherConfig) -> Result<Self> {
        fs::create_dir_all(topic_dir(&config.root)).map_err(|source| LocalStreamError::Io {
            path: topic_dir(&config.root).display().to_string(),
            source,
        })?;
        Ok(Self { config })
    }
}

pub(crate) fn publish_local(
    config: &LocalPublisherConfig,
    message: StreamMessage,
) -> Result<PublishAck> {
    validate_topic(&message.topic)?;
    validate_message_shape(&message).map_err(local_message_shape_error)?;
    let path = topic_path(&config.root, &message.topic);
    let index_path = topic_index_path(&config.root, &message.topic);
    fs::create_dir_all(topic_dir(&config.root)).map_err(|source| LocalStreamError::Io {
        path: topic_dir(&config.root).display().to_string(),
        source,
    })?;
    let topic_file_exists = path.exists();
    let mut state = recover_topic_index(&path, &index_path, config.durability)?;
    let offset = checked_offset(state.positions.len())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?;
    file.set_len(state.valid_end)
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?;
    let append_position = state.valid_end;
    write_frame(&mut file, &message)?;
    if config.durability.sync_enabled() {
        file.sync_all().map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?;
        if !topic_file_exists {
            sync_dir(&topic_dir(&config.root), config.durability)?;
        }
    }
    state.positions.push(append_position);
    write_index(&index_path, &state.positions, config.durability)?;

    Ok(PublishAck {
        topic: message.topic,
        partition: 0,
        offset,
    })
}

fn local_message_shape_error(error: StreamError) -> LocalStreamError {
    match error {
        StreamError::InvalidMessageField { field, reason } => {
            LocalStreamError::InvalidMessageField { field, reason }
        }
        other => LocalStreamError::InvalidMessageField {
            field: "message",
            reason: other.to_string(),
        },
    }
}
