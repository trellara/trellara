use std::io::{self, ErrorKind};
use std::path::Path;

#[cfg(test)]
pub(crate) use crate::index_entries::INDEX_ENTRY_BYTES;
pub(crate) use crate::index_file::{read_index, write_index};
use crate::index_scan::{scan_record_positions, scan_record_positions_from, validate_index_tail};
use crate::{LocalDurability, LocalStreamError, LocalTopicIndexStatus, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopicIndexState {
    pub(crate) positions: Vec<u64>,
    pub(crate) valid_end: u64,
    pub(crate) index_status: LocalTopicIndexStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IndexRead {
    Missing,
    Corrupt,
    Valid(Vec<u64>),
}

pub(crate) fn recover_topic_index(
    path: &Path,
    index_path: &Path,
    durability: LocalDurability,
) -> Result<TopicIndexState> {
    if !path.exists() {
        return Ok(TopicIndexState {
            positions: Vec::new(),
            valid_end: 0,
            index_status: LocalTopicIndexStatus::Healthy,
        });
    }

    let index = read_index(
        index_path,
        path.metadata()
            .map_err(|source| LocalStreamError::Io {
                path: path.display().to_string(),
                source,
            })?
            .len(),
    )?;
    let rebuild_status = match index {
        IndexRead::Valid(positions) => {
            if let Some((mut positions, valid_end)) = validate_index_tail(path, positions)? {
                let (tail_positions, tail_valid_end) = scan_record_positions_from(path, valid_end)?;
                let stale = !tail_positions.is_empty() || tail_valid_end != valid_end;
                positions.extend(tail_positions);
                if stale {
                    write_index(index_path, &positions, durability)?;
                }
                return Ok(TopicIndexState {
                    positions,
                    valid_end: tail_valid_end,
                    index_status: if stale {
                        LocalTopicIndexStatus::StaleRebuilt
                    } else {
                        LocalTopicIndexStatus::Healthy
                    },
                });
            }
            LocalTopicIndexStatus::CorruptRebuilt
        }
        IndexRead::Missing => LocalTopicIndexStatus::MissingRebuilt,
        IndexRead::Corrupt => LocalTopicIndexStatus::CorruptRebuilt,
    };

    let state = scan_record_positions(path)?;
    write_index(index_path, &state.positions, durability)?;
    Ok(TopicIndexState {
        positions: state.positions,
        valid_end: state.valid_end,
        index_status: rebuild_status,
    })
}

pub(crate) fn checked_offset(len: usize) -> Result<i64> {
    len.try_into().map_err(|source| LocalStreamError::Io {
        path: "<local-stream>".to_string(),
        source: io::Error::new(ErrorKind::InvalidData, source),
    })
}

pub(crate) fn last_valid_offset(len: usize) -> Result<Option<i64>> {
    if len == 0 {
        return Ok(None);
    }
    Ok(Some(checked_offset(len - 1)?))
}
