use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::cursor_storage::read_cursor;
use crate::paths::cursor_dir;
use crate::{validate_topic, LocalConsumerConfig, LocalCursorInspection, LocalStreamError, Result};

pub(crate) fn inspect_cursors(
    root: &Path,
    message_counts: &BTreeMap<String, i64>,
) -> Result<Vec<LocalCursorInspection>> {
    let dir = cursor_dir_root(root);
    let mut cursors = Vec::new();
    match fs::read_dir(&dir) {
        Ok(groups) => {
            for group in groups {
                let group = group.map_err(|source| LocalStreamError::Io {
                    path: dir.display().to_string(),
                    source,
                })?;
                inspect_group_entry(root, &group, message_counts, &mut cursors)?;
            }
        }
        Err(source) if source.kind() == ErrorKind::NotFound => {}
        Err(source) => {
            return Err(LocalStreamError::Io {
                path: dir.display().to_string(),
                source,
            });
        }
    }
    cursors.sort_by(|left, right| {
        left.group_id
            .cmp(&right.group_id)
            .then_with(|| left.topic.cmp(&right.topic))
    });
    Ok(cursors)
}

fn inspect_group_entry(
    root: &Path,
    group: &fs::DirEntry,
    message_counts: &BTreeMap<String, i64>,
    cursors: &mut Vec<LocalCursorInspection>,
) -> Result<()> {
    let group_path = group.path();
    if !group_path.is_dir() {
        return Ok(());
    }
    let Some(group_id) = group_path.file_name().and_then(|value| value.to_str()) else {
        return Ok(());
    };
    validate_topic(group_id)?;
    inspect_group_cursors(root, group_id, message_counts, cursors)
}

fn inspect_group_cursors(
    root: &Path,
    group_id: &str,
    message_counts: &BTreeMap<String, i64>,
    cursors: &mut Vec<LocalCursorInspection>,
) -> Result<()> {
    let dir = cursor_dir(root, group_id);
    for entry in fs::read_dir(&dir).map_err(|source| LocalStreamError::Io {
        path: dir.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| LocalStreamError::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("cursor") {
            continue;
        }
        let Some(topic) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        validate_topic(topic)?;
        let config = LocalConsumerConfig::new(root, group_id, vec![topic.to_string()]);
        cursors.push(LocalCursorInspection::from_topic_depth(
            group_id,
            topic,
            read_cursor(&config, topic)?,
            message_counts.get(topic).copied(),
        ));
    }
    Ok(())
}

fn cursor_dir_root(root: &Path) -> std::path::PathBuf {
    root.join("cursors")
}
