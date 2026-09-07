use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::native_spool::MAX_NATIVE_MESSAGE_BYTES;
use crate::native_transport::{NativeRelayTransportError, NativeRelayTransportResult};

pub(super) fn read_message(stream: &mut UnixStream) -> NativeRelayTransportResult<Vec<u8>> {
    let mut length = [0; 4];
    stream.read_exact(&mut length)?;
    let length = usize::try_from(u32::from_be_bytes(length))?;
    if length == 0 || length > MAX_NATIVE_MESSAGE_BYTES {
        return Err(NativeRelayTransportError::MessageTooLarge(length));
    }
    let mut encoded = vec![0; length];
    stream.read_exact(&mut encoded)?;
    Ok(encoded)
}

pub(super) fn write_message(
    stream: &mut UnixStream,
    bytes: &[u8],
) -> NativeRelayTransportResult<()> {
    let length = u32::try_from(bytes.len())?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(bytes)?;
    stream.flush()?;
    Ok(())
}
