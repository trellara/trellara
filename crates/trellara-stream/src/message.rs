use bytes::Bytes;
use prost::Message;
use trellara_protocol::{
    PartitionChunk, TransactionCommitMarker, TransactionEnvelope, TransactionManifest,
};

use crate::message_validation::{
    stream_partition_id, validate_chunk_payload, validate_envelope_metadata,
    validate_manifest_payload, validate_marker_payload,
};
use crate::{
    commit_marker_headers, envelope_headers, manifest_headers, partition_chunk_headers,
    partition_chunk_key, strict_chunk_headers, strict_transaction_key, Result, TopicLayout,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamHeader {
    pub key: String,
    pub value: String,
}

impl StreamHeader {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamMessage {
    pub topic: String,
    pub key: String,
    pub payload: Bytes,
    pub headers: Vec<StreamHeader>,
    pub partition: Option<i32>,
    pub position: Option<StreamPosition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamPosition {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
}

impl StreamMessage {
    pub fn strict_transaction(envelope: &TransactionEnvelope) -> Result<Self> {
        let layout = TopicLayout::new(&envelope.source_id, &envelope.dataset_id)?;
        let payload = Bytes::from(envelope.encode_checked()?);
        Ok(Self {
            topic: layout.strict_topic(),
            key: strict_transaction_key(envelope)?,
            payload,
            headers: envelope_headers(envelope),
            partition: Some(0),
            position: None,
        })
    }

    pub fn transaction_manifest(
        envelope: &TransactionEnvelope,
        manifest: &TransactionManifest,
    ) -> Result<Self> {
        validate_envelope_metadata(envelope)?;
        validate_manifest_payload(envelope, manifest)?;
        let layout = TopicLayout::new(&envelope.source_id, &envelope.dataset_id)?;
        Ok(Self {
            topic: layout.manifest_topic(),
            key: strict_transaction_key(envelope)?,
            payload: Bytes::from(manifest.encode_to_vec()),
            headers: manifest_headers(envelope, manifest),
            partition: Some(0),
            position: None,
        })
    }

    pub fn partition_chunk(envelope: &TransactionEnvelope, chunk: &PartitionChunk) -> Result<Self> {
        validate_envelope_metadata(envelope)?;
        validate_chunk_payload(envelope, chunk)?;
        let partition = stream_partition_id(chunk.partition_id)?;
        let layout = TopicLayout::new(&envelope.source_id, &envelope.dataset_id)?;
        Ok(Self {
            topic: layout.partition_topic(chunk.partition_id),
            key: partition_chunk_key(envelope, chunk.partition_id)?,
            payload: Bytes::from(chunk.encode_to_vec()),
            headers: partition_chunk_headers(envelope, chunk),
            partition: Some(partition),
            position: None,
        })
    }

    pub fn strict_chunk(envelope: &TransactionEnvelope, chunk: &PartitionChunk) -> Result<Self> {
        validate_envelope_metadata(envelope)?;
        validate_chunk_payload(envelope, chunk)?;
        let layout = TopicLayout::new(&envelope.source_id, &envelope.dataset_id)?;
        Ok(Self {
            topic: layout.strict_topic(),
            key: partition_chunk_key(envelope, chunk.partition_id)?,
            payload: Bytes::from(chunk.encode_to_vec()),
            headers: strict_chunk_headers(envelope, chunk),
            partition: Some(0),
            position: None,
        })
    }

    pub fn commit_marker(
        envelope: &TransactionEnvelope,
        marker: &TransactionCommitMarker,
    ) -> Result<Self> {
        validate_envelope_metadata(envelope)?;
        validate_marker_payload(envelope, marker)?;
        let layout = TopicLayout::new(&envelope.source_id, &envelope.dataset_id)?;
        Ok(Self {
            topic: layout.commit_topic(),
            key: strict_transaction_key(envelope)?,
            payload: Bytes::from(marker.encode_to_vec()),
            headers: commit_marker_headers(envelope, marker),
            partition: Some(0),
            position: None,
        })
    }
}
