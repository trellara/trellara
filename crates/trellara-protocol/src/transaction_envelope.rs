use prost::Message;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    envelope_validation::validate_envelope, ChangeRecord, DdlEvent, ProtocolError,
    RelationSchemaVersion, TransactionManifest, PROTOCOL_VERSION,
};

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct TransactionEnvelope {
    #[prost(uint32, tag = "1")]
    pub protocol_version: u32,
    #[prost(string, tag = "2")]
    pub source_id: String,
    #[prost(string, tag = "3")]
    pub database_id: String,
    #[prost(string, tag = "4")]
    pub dataset_id: String,
    #[prost(string, tag = "5")]
    pub transaction_id: String,
    #[prost(string, tag = "6")]
    pub begin_lsn: String,
    #[prost(string, tag = "7")]
    pub commit_lsn: String,
    #[prost(int64, tag = "8")]
    pub commit_timestamp_ms: i64,
    #[prost(message, repeated, tag = "9")]
    pub schema_versions: Vec<RelationSchemaVersion>,
    #[prost(message, repeated, tag = "10")]
    pub changes: Vec<ChangeRecord>,
    #[prost(message, optional, tag = "11")]
    pub manifest: Option<TransactionManifest>,
    #[prost(uint64, tag = "12")]
    pub checksum: u64,
    #[prost(message, repeated, tag = "13")]
    pub ddl_events: Vec<DdlEvent>,
}

#[derive(Clone, Debug)]
pub struct StrictEnvelope {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub transaction_id: String,
    pub begin_lsn: String,
    pub commit_lsn: String,
    pub commit_timestamp_ms: i64,
    pub changes: Vec<ChangeRecord>,
}

impl TransactionEnvelope {
    pub fn strict(input: StrictEnvelope) -> Self {
        let mut envelope = Self {
            protocol_version: PROTOCOL_VERSION,
            source_id: input.source_id,
            database_id: input.database_id,
            dataset_id: input.dataset_id,
            transaction_id: input.transaction_id,
            begin_lsn: input.begin_lsn,
            commit_lsn: input.commit_lsn,
            commit_timestamp_ms: input.commit_timestamp_ms,
            schema_versions: Vec::new(),
            changes: input.changes,
            manifest: None,
            checksum: 0,
            ddl_events: Vec::new(),
        };
        envelope.finalize_checksum();
        envelope
    }

    pub fn encode_checked(&self) -> Result<Vec<u8>, ProtocolError> {
        self.validate()?;
        self.verify_checksum()?;
        Ok(self.encode_to_vec())
    }

    pub fn decode_checked(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let envelope = Self::decode(bytes)?;
        envelope.verify_checksum()?;
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        validate_envelope(self)
    }

    pub fn finalize_checksum(&mut self) {
        self.checksum = self.compute_checksum();
    }

    pub fn dml_replay_after_ddl_barrier(&self) -> Self {
        let mut replay = self.clone();
        replay.ddl_events.clear();
        replay.finalize_checksum();
        replay
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
            Err(ProtocolError::ChecksumMismatch {
                expected: self.checksum,
                actual,
            })
        }
    }
}
