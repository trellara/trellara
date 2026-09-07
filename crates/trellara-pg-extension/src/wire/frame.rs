use super::codec::{
    authentication_tag, digest, require_secret, validate_identity, validate_payload,
};
use super::frame_decode;
use super::{
    NativeRelayWireError, NativeRelayWireFrame, DIGEST_BYTES, FRAME_FIXED_BYTES, FRAME_MAGIC,
    NATIVE_RELAY_WIRE_VERSION,
};
use crate::MAX_LOGICAL_FRAME_BYTES;

impl NativeRelayWireFrame {
    pub fn new(
        xid: u32,
        commit_lsn: u64,
        source_id: impl Into<String>,
        dataset_id: impl Into<String>,
        payload: Vec<u8>,
    ) -> Result<Self, NativeRelayWireError> {
        let source_id = source_id.into();
        let dataset_id = dataset_id.into();
        validate_identity(&source_id, "source_id")?;
        validate_identity(&dataset_id, "dataset_id")?;
        validate_payload(&payload)?;
        let payload_digest = digest(&payload);
        Ok(Self {
            xid,
            commit_lsn,
            source_id,
            dataset_id,
            payload,
            payload_digest,
        })
    }

    pub fn encode_authenticated(&self, secret: &[u8]) -> Result<Vec<u8>, NativeRelayWireError> {
        require_secret(secret)?;
        validate_identity(&self.source_id, "source_id")?;
        validate_identity(&self.dataset_id, "dataset_id")?;
        validate_payload(&self.payload)?;
        if digest(&self.payload) != self.payload_digest {
            return Err(NativeRelayWireError::PayloadDigestMismatch);
        }
        let source_len = checked_identity_length(&self.source_id, "source_id")?;
        let dataset_len = checked_identity_length(&self.dataset_id, "dataset_id")?;
        let payload_len = u32::try_from(self.payload.len()).map_err(|_| {
            NativeRelayWireError::PayloadTooLarge {
                length: self.payload.len(),
                max_length: MAX_LOGICAL_FRAME_BYTES,
            }
        })?;
        let mut bytes = Vec::with_capacity(
            FRAME_FIXED_BYTES
                + self.source_id.len()
                + self.dataset_id.len()
                + self.payload.len()
                + DIGEST_BYTES,
        );
        bytes.extend_from_slice(FRAME_MAGIC);
        bytes.extend_from_slice(&NATIVE_RELAY_WIRE_VERSION.to_be_bytes());
        bytes.extend_from_slice(&self.xid.to_be_bytes());
        bytes.extend_from_slice(&self.commit_lsn.to_be_bytes());
        bytes.extend_from_slice(&source_len.to_be_bytes());
        bytes.extend_from_slice(&dataset_len.to_be_bytes());
        bytes.extend_from_slice(&payload_len.to_be_bytes());
        bytes.extend_from_slice(&self.payload_digest);
        bytes.extend_from_slice(self.source_id.as_bytes());
        bytes.extend_from_slice(self.dataset_id.as_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(&authentication_tag(secret, &bytes));
        Ok(bytes)
    }

    pub fn decode_authenticated(bytes: &[u8], secret: &[u8]) -> Result<Self, NativeRelayWireError> {
        frame_decode::decode_authenticated(bytes, secret)
    }
}

fn checked_identity_length(value: &str, field: &'static str) -> Result<u16, NativeRelayWireError> {
    u16::try_from(value.len()).map_err(|_| NativeRelayWireError::IdentityTooLong {
        field,
        length: value.len(),
    })
}
