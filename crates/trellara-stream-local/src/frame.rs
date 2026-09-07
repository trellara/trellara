use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use bytes::Bytes;
use trellara_stream::{StreamHeader, StreamMessage};

use crate::frame_body::{read_legacy_frame, read_record_body, write_record_body};
use crate::frame_checksum::{frame_checksum, verify_frame_checksum};
use crate::frame_io::{read_exact_or_none, read_u32, write_u32};
use crate::frame_limits::{checked_frame_body_len, checked_write_len};
use crate::{LocalStreamError, Result};

pub(crate) const FRAME_MAGIC: &[u8; 4] = b"TLG2";
pub(crate) const LEGACY_FRAME_MAGIC: &[u8; 4] = b"TLG1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalRecord {
    pub(crate) key: String,
    pub(crate) payload: Bytes,
    pub(crate) headers: Vec<StreamHeader>,
}

impl LocalRecord {
    pub(crate) fn into_message(self, topic: String) -> StreamMessage {
        StreamMessage {
            topic,
            key: self.key,
            payload: self.payload,
            headers: self.headers,
            partition: Some(0),
            position: None,
        }
    }
}

pub(crate) fn write_frame(file: &mut File, message: &StreamMessage) -> Result<()> {
    let body = write_record_body(message)?;
    let checksum = frame_checksum(&body);

    file.write_all(FRAME_MAGIC)
        .map_err(|source| LocalStreamError::Io {
            path: "<local-stream>".to_string(),
            source,
        })?;
    write_u32(file, checked_write_len("frame_body", body.len())?)?;
    file.write_all(&body)
        .map_err(|source| LocalStreamError::Io {
            path: "<local-stream>".to_string(),
            source,
        })?;
    write_u32(file, checksum)?;
    Ok(())
}

pub(crate) fn read_frame(reader: &mut impl Read, path: &Path) -> Result<Option<LocalRecord>> {
    let mut magic = [0; 4];
    if !read_exact_or_none(reader, path, &mut magic)? {
        return Ok(None);
    }
    if &magic == LEGACY_FRAME_MAGIC {
        return read_legacy_frame(reader, path);
    }
    if &magic != FRAME_MAGIC {
        return Err(LocalStreamError::CorruptFrame {
            path: path.display().to_string(),
        });
    }

    let Some(body_len) = read_u32(reader, path)? else {
        return Ok(None);
    };
    let body_len = checked_frame_body_len(path, body_len)?;
    let mut body = vec![0; body_len];
    if !read_exact_or_none(reader, path, &mut body)? {
        return Ok(None);
    }
    let Some(expected_checksum) = read_u32(reader, path)? else {
        return Ok(None);
    };
    verify_frame_checksum(path, &body, expected_checksum)?;
    let mut body_reader = &body[..];
    let record = read_record_body(&mut body_reader, path)?;
    if !body_reader.is_empty() {
        return Err(LocalStreamError::CorruptFrame {
            path: path.display().to_string(),
        });
    }
    Ok(Some(record))
}

#[cfg(test)]
#[path = "tests/tests_frame.rs"]
mod tests;
