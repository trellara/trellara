use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use crate::native_spool::MAX_NATIVE_MESSAGE_BYTES;
use crate::native_transport::{NativeRelayTransportError, NativeRelayTransportResult};

pub(super) fn checked_message_length(length: usize) -> NativeRelayTransportResult<u32> {
    if length == 0 || length > MAX_NATIVE_MESSAGE_BYTES {
        return Err(NativeRelayTransportError::MessageTooLarge(length));
    }
    Ok(u32::try_from(length)?)
}

pub(super) fn read_record_length(
    file: &mut File,
    start: u64,
) -> NativeRelayTransportResult<Option<u32>> {
    let mut bytes = [0; 4];
    match file.read_exact(&mut bytes) {
        Ok(()) => Ok(Some(u32::from_be_bytes(bytes))),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
            if file.stream_position()? != start {
                truncate_partial_record(file, start)?;
            }
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

pub(super) fn truncate_partial_record(
    file: &mut File,
    start: u64,
) -> NativeRelayTransportResult<()> {
    file.set_len(start)?;
    file.sync_all()?;
    file.seek(SeekFrom::Start(start))?;
    Ok(())
}

pub(super) fn sync_parent(path: &Path) -> NativeRelayTransportResult<()> {
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
