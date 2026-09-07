use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

use crate::frame_limits::{checked_record_field_len, checked_write_len};
use crate::{LocalStreamError, Result};

pub(crate) fn write_bytes_to_vec(
    buffer: &mut Vec<u8>,
    field: &'static str,
    bytes: &[u8],
) -> Result<()> {
    write_u32_to_vec(buffer, checked_write_len(field, bytes.len())?);
    buffer.extend_from_slice(bytes);
    Ok(())
}

pub(crate) fn read_bytes(reader: &mut impl Read, path: &Path) -> Result<Option<Vec<u8>>> {
    let Some(len) = read_u32(reader, path)? else {
        return Ok(None);
    };
    let len = checked_record_field_len(path, len)?;
    let mut bytes = vec![0; len];
    if read_exact_or_none(reader, path, &mut bytes)? {
        Ok(Some(bytes))
    } else {
        Ok(None)
    }
}

pub(crate) fn write_u32(file: &mut File, value: u32) -> Result<()> {
    file.write_all(&value.to_le_bytes())
        .map_err(|source| LocalStreamError::Io {
            path: "<local-stream>".to_string(),
            source,
        })
}

pub(crate) fn write_u32_to_vec(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn read_u32(reader: &mut impl Read, path: &Path) -> Result<Option<u32>> {
    let mut bytes = [0; 4];
    if read_exact_or_none(reader, path, &mut bytes)? {
        Ok(Some(u32::from_le_bytes(bytes)))
    } else {
        Ok(None)
    }
}

pub(crate) fn read_exact_or_none(
    reader: &mut impl Read,
    path: &Path,
    bytes: &mut [u8],
) -> Result<bool> {
    match reader.read_exact(bytes) {
        Ok(()) => Ok(true),
        Err(source) if source.kind() == ErrorKind::UnexpectedEof => Ok(false),
        Err(source) => Err(LocalStreamError::Io {
            path: path.display().to_string(),
            source,
        }),
    }
}
