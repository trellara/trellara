use crate::native_transport::{NativeRelayTransportError, NativeRelayTransportResult};

pub(crate) const MAGIC: &[u8; 4] = b"TRKP";
pub(crate) const VERSION: u16 = 1;
pub(crate) const DIGEST_BYTES: usize = 32;
pub(crate) const FIXED_BYTES: usize = 4 + 2 + 2 + 2 + 2 + 2 + 8 + 4 + 8 + DIGEST_BYTES;

pub(crate) fn checked_len(value: &str) -> NativeRelayTransportResult<u16> {
    u16::try_from(value.len()).map_err(NativeRelayTransportError::Integer)
}

pub(crate) fn decode_strings(
    bytes: &[u8],
    mut offset: usize,
    lengths: [usize; 4],
) -> NativeRelayTransportResult<[String; 4]> {
    let mut values = Vec::with_capacity(4);
    for length in lengths {
        let value = std::str::from_utf8(slice(bytes, offset, length)?)
            .map_err(|_| NativeRelayTransportError::PublishProofCorrupt("invalid UTF-8"))?;
        values.push(value.to_owned());
        offset += length;
    }
    values.try_into().map_err(|_| {
        NativeRelayTransportError::PublishProofCorrupt("invalid publish proof strings")
    })
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> NativeRelayTransportResult<u16> {
    Ok(u16::from_be_bytes(fixed(bytes, offset)?))
}

pub(crate) fn read_u64(bytes: &[u8], offset: usize) -> NativeRelayTransportResult<u64> {
    Ok(u64::from_be_bytes(fixed(bytes, offset)?))
}

pub(crate) fn read_i32(bytes: &[u8], offset: usize) -> NativeRelayTransportResult<i32> {
    Ok(i32::from_be_bytes(fixed(bytes, offset)?))
}

pub(crate) fn read_i64(bytes: &[u8], offset: usize) -> NativeRelayTransportResult<i64> {
    Ok(i64::from_be_bytes(fixed(bytes, offset)?))
}

pub(crate) fn fixed<const N: usize>(
    bytes: &[u8],
    offset: usize,
) -> NativeRelayTransportResult<[u8; N]> {
    slice(bytes, offset, N)?
        .try_into()
        .map_err(|_| NativeRelayTransportError::PublishProofCorrupt("invalid fixed-width field"))
}

pub(crate) fn slice(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> NativeRelayTransportResult<&[u8]> {
    bytes.get(offset..offset.saturating_add(length)).ok_or(
        NativeRelayTransportError::PublishProofCorrupt("truncated publish proof"),
    )
}

pub(crate) fn corrupt<T>(reason: &'static str) -> NativeRelayTransportResult<T> {
    Err(NativeRelayTransportError::PublishProofCorrupt(reason))
}
