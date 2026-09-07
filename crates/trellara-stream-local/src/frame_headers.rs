use std::io::{self, ErrorKind, Read};
use std::path::Path;

use trellara_stream::StreamHeader;

use crate::frame_io::read_bytes;
use crate::{LocalStreamError, Result};

pub(crate) const MAX_RECORD_HEADERS: u32 = 4096;

pub(crate) fn read_required_headers(
    reader: &mut impl Read,
    path: &Path,
    header_count: u32,
) -> Result<Vec<StreamHeader>> {
    read_headers(reader, path, header_count, MissingHeaderPolicy::Corrupt)?
        .ok_or_else(|| corrupt_frame(path))
}

pub(crate) fn read_optional_headers(
    reader: &mut impl Read,
    path: &Path,
    header_count: u32,
) -> Result<Option<Vec<StreamHeader>>> {
    read_headers(reader, path, header_count, MissingHeaderPolicy::Incomplete)
}

pub(crate) fn decode_utf8_field(path: &Path, bytes: Vec<u8>) -> Result<String> {
    String::from_utf8(bytes).map_err(|source| LocalStreamError::Io {
        path: path.display().to_string(),
        source: io::Error::new(ErrorKind::InvalidData, source),
    })
}

fn read_headers(
    reader: &mut impl Read,
    path: &Path,
    header_count: u32,
    missing_policy: MissingHeaderPolicy,
) -> Result<Option<Vec<StreamHeader>>> {
    let header_count = checked_header_count(path, header_count)?;
    let mut headers = Vec::with_capacity(header_count);
    for _ in 0..header_count {
        let Some(key) = read_header_field(reader, path, missing_policy)? else {
            return Ok(None);
        };
        let Some(value) = read_header_field(reader, path, missing_policy)? else {
            return Ok(None);
        };
        headers.push(StreamHeader::new(key, value));
    }
    Ok(Some(headers))
}

fn checked_header_count(path: &Path, header_count: u32) -> Result<usize> {
    if header_count > MAX_RECORD_HEADERS {
        return Err(corrupt_frame(path));
    }
    usize::try_from(header_count).map_err(|_| LocalStreamError::FieldTooLarge { field: "headers" })
}

fn read_header_field(
    reader: &mut impl Read,
    path: &Path,
    missing_policy: MissingHeaderPolicy,
) -> Result<Option<String>> {
    match read_bytes(reader, path)? {
        Some(bytes) => decode_utf8_field(path, bytes).map(Some),
        None if missing_policy == MissingHeaderPolicy::Incomplete => Ok(None),
        None => Err(corrupt_frame(path)),
    }
}

fn corrupt_frame(path: &Path) -> LocalStreamError {
    LocalStreamError::CorruptFrame {
        path: path.display().to_string(),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MissingHeaderPolicy {
    Corrupt,
    Incomplete,
}
