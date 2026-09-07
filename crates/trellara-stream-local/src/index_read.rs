use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;

use trellara_stream::StreamMessage;

use crate::frame::read_frame;
use crate::index::recover_topic_index;
use crate::{LocalDurability, LocalStreamError, Result};

pub(crate) fn read_record_at(
    path: &Path,
    index_path: &Path,
    topic: &str,
    offset: i64,
    durability: LocalDurability,
) -> Result<Option<StreamMessage>> {
    if offset < 0 {
        return Err(LocalStreamError::NegativeCursorOffset { offset });
    }
    let state = recover_topic_index(path, index_path, durability)?;
    let Some(position) = usize::try_from(offset)
        .ok()
        .and_then(|offset| state.positions.get(offset))
    else {
        return Ok(None);
    };
    let mut reader = BufReader::new(File::open(path).map_err(|source| LocalStreamError::Io {
        path: path.display().to_string(),
        source,
    })?);
    reader
        .seek(SeekFrom::Start(*position))
        .map_err(|source| LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        })?;
    Ok(read_frame(&mut reader, path)?.map(|record| record.into_message(topic.to_string())))
}
