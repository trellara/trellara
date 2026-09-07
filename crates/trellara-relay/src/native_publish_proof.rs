use sha2::{Digest, Sha256};
use trellara_pg_extension::NativeRelayWireFrame;

use crate::native_publish_proof_codec::{
    checked_len, corrupt, decode_strings, read_i32, read_i64, read_u16, read_u64, slice,
    DIGEST_BYTES, FIXED_BYTES, MAGIC, VERSION,
};
use crate::native_transport::{NativeRelayTransportError, NativeRelayTransportResult};

pub(crate) const MAX_PUBLISH_PROOF_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct NativeFrameKey {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) commit_lsn: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativePublishDestination {
    pub(crate) cluster_id: String,
    pub(crate) topic: String,
    pub(crate) partition: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeKafkaPublishProof {
    pub(crate) key: NativeFrameKey,
    pub(crate) payload_digest: [u8; DIGEST_BYTES],
    pub(crate) destination: NativePublishDestination,
    pub(crate) offset: i64,
}

impl NativeKafkaPublishProof {
    pub(crate) fn new(
        frame: &NativeRelayWireFrame,
        destination: NativePublishDestination,
        offset: i64,
    ) -> NativeRelayTransportResult<Self> {
        if offset < 0 {
            return Err(NativeRelayTransportError::InvalidKafkaOffset(offset));
        }
        Ok(Self {
            key: frame_key(frame),
            payload_digest: frame.payload_digest,
            destination,
            offset,
        })
    }

    pub(crate) fn encode(&self) -> NativeRelayTransportResult<Vec<u8>> {
        let source_len = checked_len(&self.key.source_id)?;
        let dataset_len = checked_len(&self.key.dataset_id)?;
        let cluster_len = checked_len(&self.destination.cluster_id)?;
        let topic_len = checked_len(&self.destination.topic)?;
        let mut bytes = Vec::with_capacity(FIXED_BYTES + DIGEST_BYTES + 256);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&VERSION.to_be_bytes());
        bytes.extend_from_slice(&source_len.to_be_bytes());
        bytes.extend_from_slice(&dataset_len.to_be_bytes());
        bytes.extend_from_slice(&cluster_len.to_be_bytes());
        bytes.extend_from_slice(&topic_len.to_be_bytes());
        bytes.extend_from_slice(&self.key.commit_lsn.to_be_bytes());
        bytes.extend_from_slice(&self.destination.partition.to_be_bytes());
        bytes.extend_from_slice(&self.offset.to_be_bytes());
        bytes.extend_from_slice(&self.payload_digest);
        bytes.extend_from_slice(self.key.source_id.as_bytes());
        bytes.extend_from_slice(self.key.dataset_id.as_bytes());
        bytes.extend_from_slice(self.destination.cluster_id.as_bytes());
        bytes.extend_from_slice(self.destination.topic.as_bytes());
        let checksum: [u8; DIGEST_BYTES] = Sha256::digest(&bytes).into();
        bytes.extend_from_slice(&checksum);
        if bytes.len() > MAX_PUBLISH_PROOF_BYTES {
            return Err(NativeRelayTransportError::PublishProofCorrupt(
                "publish proof exceeds its size bound",
            ));
        }
        Ok(bytes)
    }

    pub(crate) fn decode(bytes: &[u8]) -> NativeRelayTransportResult<Self> {
        if bytes.len() < FIXED_BYTES + DIGEST_BYTES || bytes.get(..4) != Some(MAGIC) {
            return corrupt("invalid publish proof prefix");
        }
        if read_u16(bytes, 4)? != VERSION {
            return corrupt("unsupported publish proof version");
        }
        let lengths = [
            usize::from(read_u16(bytes, 6)?),
            usize::from(read_u16(bytes, 8)?),
            usize::from(read_u16(bytes, 10)?),
            usize::from(read_u16(bytes, 12)?),
        ];
        let content_end = lengths.iter().try_fold(FIXED_BYTES, |offset, length| {
            offset.checked_add(*length).ok_or(())
        });
        let content_end = content_end.map_err(|()| {
            NativeRelayTransportError::PublishProofCorrupt("publish proof length overflow")
        })?;
        if content_end.checked_add(DIGEST_BYTES) != Some(bytes.len()) {
            return corrupt("publish proof length mismatch");
        }
        let expected: [u8; DIGEST_BYTES] = Sha256::digest(&bytes[..content_end]).into();
        if bytes[content_end..] != expected {
            return corrupt("publish proof checksum mismatch");
        }
        let mut digest = [0; DIGEST_BYTES];
        digest.copy_from_slice(slice(bytes, 34, DIGEST_BYTES)?);
        let strings = decode_strings(bytes, FIXED_BYTES, lengths)?;
        let offset = read_i64(bytes, 26)?;
        if offset < 0 {
            return Err(NativeRelayTransportError::InvalidKafkaOffset(offset));
        }
        Ok(Self {
            key: NativeFrameKey {
                source_id: strings[0].clone(),
                dataset_id: strings[1].clone(),
                commit_lsn: read_u64(bytes, 14)?,
            },
            payload_digest: digest,
            destination: NativePublishDestination {
                cluster_id: strings[2].clone(),
                topic: strings[3].clone(),
                partition: read_i32(bytes, 22)?,
            },
            offset,
        })
    }
}

pub(crate) fn frame_key(frame: &NativeRelayWireFrame) -> NativeFrameKey {
    NativeFrameKey {
        source_id: frame.source_id.clone(),
        dataset_id: frame.dataset_id.clone(),
        commit_lsn: frame.commit_lsn,
    }
}
