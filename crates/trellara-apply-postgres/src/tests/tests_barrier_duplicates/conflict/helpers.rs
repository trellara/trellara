use super::*;

pub(crate) fn unlisted_chunk_message(
    envelope: &TransactionEnvelope,
    partition_id: u32,
) -> StreamMessage {
    let chunk = PartitionChunk::new(
        envelope.transaction_id.clone(),
        partition_id,
        envelope.changes.clone(),
    );
    StreamMessage::partition_chunk(envelope, &chunk).expect("unlisted chunk message")
}

pub(crate) fn unlisted_partition_id(manifest: &TransactionManifest) -> u32 {
    manifest
        .partitions
        .iter()
        .map(|partition| partition.id)
        .max()
        .expect("manifest partition")
        + 1
}

pub(crate) fn assert_partition_not_in_manifest(
    error: ApplyWorkerError,
    expected_partition_id: u32,
) {
    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::PartitionNotInManifest {
            partition_id,
            ..
        }) if partition_id == expected_partition_id
    ));
}
