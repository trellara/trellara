use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;

use crate::frame::read_frame;
use crate::index::TopicIndexState;
use crate::{LocalStreamError, LocalTopicIndexStatus, Result};

pub(crate) fn validate_index_tail(
    path: &Path,
    positions: Vec<u64>,
) -> Result<Option<(Vec<u64>, u64)>> {
    let Some(last_position) = positions.last().copied() else {
        let file_bytes = path
            .metadata()
            .map_err(|source| LocalStreamError::Io {
                path: path.display().to_string(),
                source,
            })?
            .len();
        return Ok((file_bytes == 0).then_some((positions, 0)));
    };

    let mut reader = BufReader::new(File::open(path).map_err(|source| LocalStreamError::Io {
        path: path.display().to_string(),
        source,
    })?);
    reader
        .seek(SeekFrom::Start(last_position))
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?;
    let Some(_) = read_frame(&mut reader, path)? else {
        return Ok(None);
    };
    let valid_end = reader
        .stream_position()
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?;
    Ok(Some((positions, valid_end)))
}

pub(crate) fn scan_record_positions(path: &Path) -> Result<TopicIndexState> {
    if !path.exists() {
        return Ok(TopicIndexState {
            positions: Vec::new(),
            valid_end: 0,
            index_status: LocalTopicIndexStatus::Healthy,
        });
    }
    let (positions, valid_end) = scan_record_positions_from(path, 0)?;
    Ok(TopicIndexState {
        positions,
        valid_end,
        index_status: LocalTopicIndexStatus::Healthy,
    })
}

pub(crate) fn scan_record_positions_from(path: &Path, start: u64) -> Result<(Vec<u64>, u64)> {
    let mut reader = BufReader::new(File::open(path).map_err(|source| LocalStreamError::Io {
        path: path.display().to_string(),
        source,
    })?);
    reader
        .seek(SeekFrom::Start(start))
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?;
    let mut positions = Vec::new();
    let mut valid_end = start;
    loop {
        let position = reader
            .stream_position()
            .map_err(|source| LocalStreamError::Io {
                path: path.display().to_string(),
                source,
            })?;
        match read_frame(&mut reader, path)? {
            Some(_) => {
                positions.push(position);
                valid_end = reader
                    .stream_position()
                    .map_err(|source| LocalStreamError::Io {
                        path: path.display().to_string(),
                        source,
                    })?;
            }
            None => return Ok((positions, valid_end)),
        }
    }
}
