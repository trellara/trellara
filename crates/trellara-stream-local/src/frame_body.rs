use std::io::Read;
use std::path::Path;

use bytes::Bytes;
use trellara_stream::StreamMessage;

use crate::frame::LocalRecord;
use crate::frame_headers::{
    decode_utf8_field, read_optional_headers, read_required_headers, MAX_RECORD_HEADERS,
};
use crate::frame_io::{read_bytes, read_u32, write_bytes_to_vec, write_u32_to_vec};
use crate::frame_limits::checked_write_len;
use crate::{LocalStreamError, Result};

pub(crate) fn write_record_body(message: &StreamMessage) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    write_bytes_to_vec(&mut body, "key", message.key.as_bytes())?;
    write_bytes_to_vec(&mut body, "payload", &message.payload)?;
    let header_count = checked_record_header_len(message.headers.len())?;
    write_u32_to_vec(&mut body, header_count);
    for header in &message.headers {
        write_bytes_to_vec(&mut body, "header_key", header.key.as_bytes())?;
        write_bytes_to_vec(&mut body, "header_value", header.value.as_bytes())?;
    }
    Ok(body)
}

pub(crate) fn read_legacy_frame(
    reader: &mut impl Read,
    path: &Path,
) -> Result<Option<LocalRecord>> {
    let Some(key) = read_bytes(reader, path)? else {
        return Ok(None);
    };
    let key = decode_utf8_field(path, key)?;
    let Some(payload) = read_bytes(reader, path)? else {
        return Ok(None);
    };
    let payload = Bytes::from(payload);
    let Some(header_count) = read_u32(reader, path)? else {
        return Ok(None);
    };
    let Some(headers) = read_optional_headers(reader, path, header_count)? else {
        return Ok(None);
    };
    Ok(Some(LocalRecord {
        key,
        payload,
        headers,
    }))
}

pub(crate) fn read_record_body(reader: &mut impl Read, path: &Path) -> Result<LocalRecord> {
    let Some(key) = read_bytes(reader, path)? else {
        return Err(LocalStreamError::CorruptFrame {
            path: path.display().to_string(),
        });
    };
    let key = decode_utf8_field(path, key)?;
    let Some(payload) = read_bytes(reader, path)? else {
        return Err(LocalStreamError::CorruptFrame {
            path: path.display().to_string(),
        });
    };
    let payload = Bytes::from(payload);
    let Some(header_count) = read_u32(reader, path)? else {
        return Err(LocalStreamError::CorruptFrame {
            path: path.display().to_string(),
        });
    };
    let headers = read_required_headers(reader, path, header_count)?;
    Ok(LocalRecord {
        key,
        payload,
        headers,
    })
}

fn checked_record_header_len(header_count: usize) -> Result<u32> {
    let header_count = checked_write_len("headers", header_count)?;
    if header_count > MAX_RECORD_HEADERS {
        Err(LocalStreamError::FieldTooLarge { field: "headers" })
    } else {
        Ok(header_count)
    }
}

#[cfg(test)]
#[path = "tests/tests_frame_body.rs"]
mod tests;
