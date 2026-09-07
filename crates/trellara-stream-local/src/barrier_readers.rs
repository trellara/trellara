use std::path::Path;

use prost::Message;
use trellara_protocol::{
    validate_commit_marker, validate_transaction_manifest, PartitionChunk, TransactionCommitMarker,
    TransactionManifest,
};

use crate::barrier_headers::{
    validate_chunk_headers, validate_commit_marker_headers, validate_manifest_headers,
};
use crate::{read_local_message_at, LocalStreamError, Result};

pub(crate) fn read_manifest(root: &Path, topic: &str, offset: i64) -> Result<TransactionManifest> {
    let message = read_required(root, topic, offset)?;
    let manifest = TransactionManifest::decode(message.payload.as_ref()).map_err(|source| {
        LocalStreamError::BarrierDecode {
            message_kind: "transaction manifest",
            topic: topic.to_string(),
            offset,
            source,
        }
    })?;
    validate_manifest_headers(&message, &manifest, offset)?;
    validate_transaction_manifest(&manifest)?;
    Ok(manifest)
}

pub(crate) fn read_commit_marker(
    root: &Path,
    topic: &str,
    offset: i64,
) -> Result<TransactionCommitMarker> {
    let message = read_required(root, topic, offset)?;
    let marker = TransactionCommitMarker::decode(message.payload.as_ref()).map_err(|source| {
        LocalStreamError::BarrierDecode {
            message_kind: "transaction commit marker",
            topic: topic.to_string(),
            offset,
            source,
        }
    })?;
    validate_commit_marker_headers(&message, &marker, offset)?;
    validate_commit_marker(&marker)?;
    Ok(marker)
}

pub(crate) fn read_partition_chunk(
    root: &Path,
    topic: &str,
    expected_partition_id: u32,
    offset: i64,
) -> Result<PartitionChunk> {
    let message = read_required(root, topic, offset)?;
    let chunk = PartitionChunk::decode(message.payload.as_ref()).map_err(|source| {
        LocalStreamError::BarrierDecode {
            message_kind: "partition chunk",
            topic: topic.to_string(),
            offset,
            source,
        }
    })?;
    if chunk.partition_id != expected_partition_id {
        return Err(LocalStreamError::BarrierPartitionMismatch {
            topic: topic.to_string(),
            offset,
            expected_partition_id,
            actual_partition_id: chunk.partition_id,
        });
    }
    validate_chunk_headers(&message, &chunk, offset)?;
    Ok(chunk)
}

fn read_required(root: &Path, topic: &str, offset: i64) -> Result<trellara_stream::StreamMessage> {
    read_local_message_at(root, topic, offset)?.ok_or_else(|| {
        LocalStreamError::MissingBarrierMessage {
            topic: topic.to_string(),
            offset,
        }
    })
}
