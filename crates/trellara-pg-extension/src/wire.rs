use std::fmt;

pub const NATIVE_RELAY_WIRE_VERSION: u16 = 1;
pub(super) const FRAME_MAGIC: &[u8; 4] = b"TRNF";
pub(super) const ACK_MAGIC: &[u8; 4] = b"TRNA";
pub(super) const DIGEST_BYTES: usize = 32;
pub(super) const FRAME_FIXED_BYTES: usize = 4 + 2 + 4 + 8 + 2 + 2 + 4 + DIGEST_BYTES;
pub(super) const ACK_UNSIGNED_BYTES: usize = 4 + 2 + 8 + DIGEST_BYTES;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeRelayWireFrame {
    pub xid: u32,
    pub commit_lsn: u64,
    pub source_id: String,
    pub dataset_id: String,
    pub payload: Vec<u8>,
    pub payload_digest: [u8; DIGEST_BYTES],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeRelayWireAck {
    pub commit_lsn: u64,
    pub payload_digest: [u8; DIGEST_BYTES],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeRelayWireError {
    EmptySecret,
    EmptyIdentity { field: &'static str },
    IdentityTooLong { field: &'static str, length: usize },
    EmptyPayload,
    PayloadTooLarge { length: usize, max_length: usize },
    Truncated,
    InvalidMagic,
    UnsupportedVersion(u16),
    InvalidLength,
    PayloadDigestMismatch,
    AuthenticationFailed,
    InvalidUtf8 { field: &'static str },
}

impl fmt::Display for NativeRelayWireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySecret => formatter.write_str("native relay secret cannot be empty"),
            Self::EmptyIdentity { field } => write!(formatter, "{field} cannot be empty"),
            Self::IdentityTooLong { field, length } => {
                write!(formatter, "{field} is {length} bytes; maximum is u16::MAX")
            }
            Self::EmptyPayload => formatter.write_str("native relay payload cannot be empty"),
            Self::PayloadTooLarge { length, max_length } => write!(
                formatter,
                "native relay payload is {length} bytes; maximum is {max_length}"
            ),
            Self::Truncated => formatter.write_str("native relay message is truncated"),
            Self::InvalidMagic => formatter.write_str("native relay message has invalid magic"),
            Self::UnsupportedVersion(version) => {
                write!(
                    formatter,
                    "native relay wire version {version} is unsupported"
                )
            }
            Self::InvalidLength => formatter.write_str("native relay message has invalid length"),
            Self::PayloadDigestMismatch => {
                formatter.write_str("native relay payload digest does not match")
            }
            Self::AuthenticationFailed => {
                formatter.write_str("native relay message authentication failed")
            }
            Self::InvalidUtf8 { field } => write!(formatter, "{field} is not valid UTF-8"),
        }
    }
}

impl std::error::Error for NativeRelayWireError {}

mod ack;
mod codec;
mod frame;
mod frame_decode;

pub use ack::native_relay_ack;
