use super::codec::{
    authentication_tag, constant_time_equal, digest, read_u16, read_u32, read_u64, require_secret,
    slice, validate_identity,
};
use super::{
    NativeRelayWireError, NativeRelayWireFrame, DIGEST_BYTES, FRAME_FIXED_BYTES, FRAME_MAGIC,
    NATIVE_RELAY_WIRE_VERSION,
};
use crate::MAX_LOGICAL_FRAME_BYTES;

pub(super) fn decode_authenticated(
    bytes: &[u8],
    secret: &[u8],
) -> Result<NativeRelayWireFrame, NativeRelayWireError> {
    require_secret(secret)?;
    validate_frame_prefix(bytes)?;
    let xid = read_u32(bytes, 6)?;
    let commit_lsn = read_u64(bytes, 10)?;
    let source_len = usize::from(read_u16(bytes, 18)?);
    let dataset_len = usize::from(read_u16(bytes, 20)?);
    let payload_len =
        usize::try_from(read_u32(bytes, 22)?).map_err(|_| NativeRelayWireError::InvalidLength)?;
    if payload_len > MAX_LOGICAL_FRAME_BYTES {
        return Err(NativeRelayWireError::PayloadTooLarge {
            length: payload_len,
            max_length: MAX_LOGICAL_FRAME_BYTES,
        });
    }
    let content_end = checked_content_end(source_len, dataset_len, payload_len)?;
    validate_frame_authentication(bytes, content_end, secret)?;
    let mut payload_digest = [0; DIGEST_BYTES];
    payload_digest.copy_from_slice(slice(bytes, 26, DIGEST_BYTES)?);
    let source_start = FRAME_FIXED_BYTES;
    let dataset_start = source_start + source_len;
    let payload_start = dataset_start + dataset_len;
    let source_id = decode_identity(bytes, source_start, source_len, "source_id")?;
    let dataset_id = decode_identity(bytes, dataset_start, dataset_len, "dataset_id")?;
    let payload = slice(bytes, payload_start, payload_len)?.to_vec();
    if digest(&payload) != payload_digest {
        return Err(NativeRelayWireError::PayloadDigestMismatch);
    }
    Ok(NativeRelayWireFrame {
        xid,
        commit_lsn,
        source_id,
        dataset_id,
        payload,
        payload_digest,
    })
}

fn validate_frame_prefix(bytes: &[u8]) -> Result<(), NativeRelayWireError> {
    if bytes.len() < FRAME_FIXED_BYTES + DIGEST_BYTES {
        return Err(NativeRelayWireError::Truncated);
    }
    if &bytes[..4] != FRAME_MAGIC {
        return Err(NativeRelayWireError::InvalidMagic);
    }
    let version = read_u16(bytes, 4)?;
    if version != NATIVE_RELAY_WIRE_VERSION {
        return Err(NativeRelayWireError::UnsupportedVersion(version));
    }
    Ok(())
}

fn checked_content_end(
    source_len: usize,
    dataset_len: usize,
    payload_len: usize,
) -> Result<usize, NativeRelayWireError> {
    FRAME_FIXED_BYTES
        .checked_add(source_len)
        .and_then(|offset| offset.checked_add(dataset_len))
        .and_then(|offset| offset.checked_add(payload_len))
        .ok_or(NativeRelayWireError::InvalidLength)
}

fn validate_frame_authentication(
    bytes: &[u8],
    content_end: usize,
    secret: &[u8],
) -> Result<(), NativeRelayWireError> {
    let total_end = content_end
        .checked_add(DIGEST_BYTES)
        .ok_or(NativeRelayWireError::InvalidLength)?;
    if total_end != bytes.len() {
        return Err(NativeRelayWireError::InvalidLength);
    }
    let expected_tag = authentication_tag(secret, &bytes[..content_end]);
    if !constant_time_equal(slice(bytes, content_end, DIGEST_BYTES)?, &expected_tag) {
        return Err(NativeRelayWireError::AuthenticationFailed);
    }
    Ok(())
}

fn decode_identity(
    bytes: &[u8],
    offset: usize,
    length: usize,
    field: &'static str,
) -> Result<String, NativeRelayWireError> {
    let value = std::str::from_utf8(slice(bytes, offset, length)?)
        .map_err(|_| NativeRelayWireError::InvalidUtf8 { field })?
        .to_owned();
    validate_identity(&value, field)?;
    Ok(value)
}
