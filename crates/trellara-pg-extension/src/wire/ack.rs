use super::codec::{
    authentication_tag, constant_time_equal, read_u16, read_u64, require_secret, slice,
};
use super::{
    NativeRelayWireAck, NativeRelayWireError, NativeRelayWireFrame, ACK_MAGIC, ACK_UNSIGNED_BYTES,
    DIGEST_BYTES, NATIVE_RELAY_WIRE_VERSION,
};

pub fn native_relay_ack(frame: &NativeRelayWireFrame) -> NativeRelayWireAck {
    NativeRelayWireAck {
        commit_lsn: frame.commit_lsn,
        payload_digest: frame.payload_digest,
    }
}

impl NativeRelayWireAck {
    pub fn encode_authenticated(&self, secret: &[u8]) -> Result<Vec<u8>, NativeRelayWireError> {
        require_secret(secret)?;
        let mut bytes = Vec::with_capacity(ACK_UNSIGNED_BYTES + DIGEST_BYTES);
        bytes.extend_from_slice(ACK_MAGIC);
        bytes.extend_from_slice(&NATIVE_RELAY_WIRE_VERSION.to_be_bytes());
        bytes.extend_from_slice(&self.commit_lsn.to_be_bytes());
        bytes.extend_from_slice(&self.payload_digest);
        let tag = authentication_tag(secret, &bytes);
        bytes.extend_from_slice(&tag);
        Ok(bytes)
    }

    pub fn decode_authenticated(bytes: &[u8], secret: &[u8]) -> Result<Self, NativeRelayWireError> {
        require_secret(secret)?;
        if bytes.len() != ACK_UNSIGNED_BYTES + DIGEST_BYTES {
            return Err(NativeRelayWireError::InvalidLength);
        }
        if &bytes[..4] != ACK_MAGIC {
            return Err(NativeRelayWireError::InvalidMagic);
        }
        let version = read_u16(bytes, 4)?;
        if version != NATIVE_RELAY_WIRE_VERSION {
            return Err(NativeRelayWireError::UnsupportedVersion(version));
        }
        let expected_tag = authentication_tag(secret, &bytes[..ACK_UNSIGNED_BYTES]);
        if !constant_time_equal(&bytes[ACK_UNSIGNED_BYTES..], &expected_tag) {
            return Err(NativeRelayWireError::AuthenticationFailed);
        }
        let mut payload_digest = [0; DIGEST_BYTES];
        payload_digest.copy_from_slice(slice(bytes, 14, DIGEST_BYTES)?);
        Ok(Self {
            commit_lsn: read_u64(bytes, 6)?,
            payload_digest,
        })
    }
}
