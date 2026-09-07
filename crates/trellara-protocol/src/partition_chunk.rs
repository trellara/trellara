use prost::Message;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::xxh3_64;

use crate::{ChangeRecord, ProtocolError};

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct PartitionChunk {
    #[prost(string, tag = "1")]
    pub transaction_id: String,
    #[prost(uint32, tag = "2")]
    pub partition_id: u32,
    #[prost(message, repeated, tag = "3")]
    pub changes: Vec<ChangeRecord>,
    #[prost(uint64, tag = "4")]
    pub checksum: u64,
}

impl PartitionChunk {
    pub fn new(
        transaction_id: impl Into<String>,
        partition_id: u32,
        changes: Vec<ChangeRecord>,
    ) -> Self {
        let mut chunk = Self {
            transaction_id: transaction_id.into(),
            partition_id,
            changes,
            checksum: 0,
        };
        chunk.finalize_checksum();
        chunk
    }

    pub fn finalize_checksum(&mut self) {
        self.checksum = self.compute_checksum();
    }

    pub fn compute_checksum(&self) -> u64 {
        let mut canonical = self.clone();
        canonical.checksum = 0;
        xxh3_64(&canonical.encode_to_vec())
    }

    pub fn verify_checksum(&self) -> Result<(), ProtocolError> {
        let actual = self.compute_checksum();
        if self.checksum == actual {
            Ok(())
        } else {
            Err(ProtocolError::PartitionChecksumMismatch {
                partition_id: self.partition_id,
                expected: self.checksum,
                actual,
            })
        }
    }
}
