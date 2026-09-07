use sha2::{Digest, Sha256};

use super::{NativeRelayWireError, DIGEST_BYTES};
use crate::MAX_LOGICAL_FRAME_BYTES;

pub(super) fn validate_identity(
    value: &str,
    field: &'static str,
) -> Result<(), NativeRelayWireError> {
    if value.is_empty() {
        return Err(NativeRelayWireError::EmptyIdentity { field });
    }
    if value.len() > usize::from(u16::MAX) {
        return Err(NativeRelayWireError::IdentityTooLong {
            field,
            length: value.len(),
        });
    }
    Ok(())
}

pub(super) fn validate_payload(payload: &[u8]) -> Result<(), NativeRelayWireError> {
    if payload.is_empty() {
        return Err(NativeRelayWireError::EmptyPayload);
    }
    if payload.len() > MAX_LOGICAL_FRAME_BYTES {
        return Err(NativeRelayWireError::PayloadTooLarge {
            length: payload.len(),
            max_length: MAX_LOGICAL_FRAME_BYTES,
        });
    }
    Ok(())
}

pub(super) fn require_secret(secret: &[u8]) -> Result<(), NativeRelayWireError> {
    if secret.is_empty() {
        Err(NativeRelayWireError::EmptySecret)
    } else {
        Ok(())
    }
}

pub(super) fn digest(bytes: &[u8]) -> [u8; DIGEST_BYTES] {
    Sha256::digest(bytes).into()
}

pub(super) fn authentication_tag(secret: &[u8], bytes: &[u8]) -> [u8; DIGEST_BYTES] {
    const BLOCK_BYTES: usize = 64;
    let mut key = [0; BLOCK_BYTES];
    if secret.len() > BLOCK_BYTES {
        key[..DIGEST_BYTES].copy_from_slice(&digest(secret));
    } else {
        key[..secret.len()].copy_from_slice(secret);
    }
    let mut inner_pad = [0x36; BLOCK_BYTES];
    let mut outer_pad = [0x5c; BLOCK_BYTES];
    for index in 0..BLOCK_BYTES {
        inner_pad[index] ^= key[index];
        outer_pad[index] ^= key[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(b"trellara-native-relay-v1\0");
    inner.update(bytes);
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}

pub(super) fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

pub(super) fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, NativeRelayWireError> {
    Ok(u16::from_be_bytes(
        slice(bytes, offset, 2)?.try_into().expect("two-byte slice"),
    ))
}

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, NativeRelayWireError> {
    Ok(u32::from_be_bytes(
        slice(bytes, offset, 4)?
            .try_into()
            .expect("four-byte slice"),
    ))
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, NativeRelayWireError> {
    Ok(u64::from_be_bytes(
        slice(bytes, offset, 8)?
            .try_into()
            .expect("eight-byte slice"),
    ))
}

pub(super) fn slice(
    bytes: &[u8],
    offset: usize,
    length: usize,
) -> Result<&[u8], NativeRelayWireError> {
    let end = offset
        .checked_add(length)
        .ok_or(NativeRelayWireError::InvalidLength)?;
    bytes
        .get(offset..end)
        .ok_or(NativeRelayWireError::Truncated)
}
