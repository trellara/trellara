use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::Path;

use crate::paths::cursor_path;
use crate::{LocalConsumerConfig, LocalDurability, LocalStreamError, Result};

pub(crate) fn read_cursor(config: &LocalConsumerConfig, topic: &str) -> Result<i64> {
    let path = cursor_path(&config.root, &config.group_id, topic);
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let value = contents.as_str();
            if value.trim() != value {
                return Err(LocalStreamError::InvalidCursor {
                    path: path.display().to_string(),
                    value: value.to_string(),
                });
            }
            let offset = value
                .parse::<i64>()
                .map_err(|_| LocalStreamError::InvalidCursor {
                    path: path.display().to_string(),
                    value: value.to_string(),
                })?;
            if offset < 0 {
                return Err(LocalStreamError::NegativeCursorOffset { offset });
            }
            Ok(offset)
        }
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(0),
        Err(source) => Err(LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        }),
    }
}

pub(crate) fn write_cursor(
    config: &LocalConsumerConfig,
    topic: &str,
    next_offset: i64,
) -> Result<()> {
    if next_offset < 0 {
        return Err(LocalStreamError::NegativeCursorOffset {
            offset: next_offset,
        });
    }
    let path = cursor_path(&config.root, &config.group_id, topic);
    let parent = parent_dir(&path)?;
    fs::create_dir_all(parent).map_err(|source| LocalStreamError::Io {
        path: parent.display().to_string(),
        source,
    })?;
    let temp = path.with_extension("tmp");
    {
        let mut file = File::create(&temp).map_err(|source| LocalStreamError::Io {
            path: temp.display().to_string(),
            source,
        })?;
        file.write_all(next_offset.to_string().as_bytes())
            .map_err(|source| LocalStreamError::Io {
                path: temp.display().to_string(),
                source,
            })?;
        if config.durability.sync_enabled() {
            file.sync_all().map_err(|source| LocalStreamError::Io {
                path: temp.display().to_string(),
                source,
            })?;
        }
    }
    fs::rename(&temp, &path).map_err(|source| LocalStreamError::Io {
        path: path.display().to_string(),
        source,
    })?;
    sync_parent_dir(&path, config.durability)
}

pub(crate) fn sync_parent_dir(path: &Path, durability: LocalDurability) -> Result<()> {
    let parent = parent_dir(path)?;
    sync_dir(parent, durability)
}

pub(crate) fn sync_dir(path: &Path, durability: LocalDurability) -> Result<()> {
    if !durability.sync_enabled() {
        return Ok(());
    }
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })
}

fn parent_dir(path: &Path) -> Result<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| LocalStreamError::MissingParentPath {
            path: path.display().to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn sync_parent_dir_rejects_path_without_parent() {
        let error = sync_parent_dir(Path::new("cursor"), LocalDurability::Buffered)
            .expect_err("missing parent");

        assert!(matches!(error, LocalStreamError::MissingParentPath { .. }));
    }
}
