use std::io;
use std::path::{Path, PathBuf};

use crate::cursor_storage::write_cursor;
use crate::index::recover_topic_index;
use crate::paths::{topic_index_path, topic_path};
use crate::{
    LocalConsumerConfig, LocalCursorInspection, LocalDurability, LocalStreamError, Result,
};

pub fn set_local_cursor(
    root: impl Into<PathBuf>,
    group_id: impl Into<String>,
    topic: impl Into<String>,
    next_offset: i64,
) -> Result<LocalCursorInspection> {
    set_local_cursor_with_policy(root, group_id, topic, next_offset, false)
}

pub fn set_local_cursor_with_policy(
    root: impl Into<PathBuf>,
    group_id: impl Into<String>,
    topic: impl Into<String>,
    next_offset: i64,
    allow_ahead: bool,
) -> Result<LocalCursorInspection> {
    if next_offset < 0 {
        return Err(LocalStreamError::NegativeCursorOffset {
            offset: next_offset,
        });
    }
    let topic = topic.into();
    let config = LocalConsumerConfig::new(root, group_id, vec![topic.clone()]);
    config.validate()?;
    if !allow_ahead {
        let message_count = local_topic_message_count(&config.root, &topic)?;
        if next_offset > message_count {
            return Err(LocalStreamError::CursorOffsetAhead {
                topic,
                offset: next_offset,
                message_count,
            });
        }
    }
    let message_count = local_topic_message_count(&config.root, &topic)?;
    write_cursor(&config, &topic, next_offset)?;
    Ok(LocalCursorInspection::from_topic_depth(
        config.group_id,
        topic,
        next_offset,
        Some(message_count),
    ))
}

fn local_topic_message_count(root: &Path, topic: &str) -> Result<i64> {
    let state = recover_topic_index(
        &topic_path(root, topic),
        &topic_index_path(root, topic),
        LocalDurability::default(),
    )?;
    i64::try_from(state.positions.len()).map_err(|source| LocalStreamError::Io {
        path: topic_index_path(root, topic).display().to_string(),
        source: io::Error::other(source),
    })
}
