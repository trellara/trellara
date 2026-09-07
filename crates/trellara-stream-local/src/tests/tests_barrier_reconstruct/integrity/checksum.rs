use super::*;
use prost::Message;
use std::fs;
use trellara_protocol::{PartitionChunk, TransactionCommitMarker};

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_corrupt_chunk_manifest_proof() {
    let root = temp_root("barrier-reconstruct-corrupt-chunk");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let manifest_ack = publisher
        .publish(StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest"))
        .await
        .expect("publish manifest");
    let commit_ack = publisher
        .publish(StreamMessage::commit_marker(&envelope, &marker).expect("commit"))
        .await
        .expect("publish commit");
    let mut partition_offsets = Vec::new();
    for chunk in &plan.chunks {
        let mut message = StreamMessage::partition_chunk(&envelope, chunk).expect("chunk");
        if partition_offsets.is_empty() {
            let mut corrupt =
                PartitionChunk::decode(message.payload.as_ref()).expect("decode chunk");
            corrupt.checksum += 1;
            replace_header(
                &mut message,
                "trellara.partition_checksum",
                &corrupt.checksum.to_string(),
            );
            message.payload = corrupt.encode_to_vec().into();
        }
        let ack = publisher.publish(message).await.expect("publish chunk");
        partition_offsets.push(LocalPartitionChunkOffset {
            partition_id: chunk.partition_id,
            offset: ack.offset,
        });
    }

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets,
        },
    )
    .expect_err("corrupt chunk");

    assert!(matches!(
        error,
        LocalStreamError::BarrierProtocol(
            trellara_protocol::ProtocolError::PartitionChecksumMismatch { .. }
        )
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_invalid_commit_marker_payload() {
    let root = temp_root("barrier-reconstruct-invalid-commit-marker");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let mut marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let manifest_ack = publisher
        .publish(StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest"))
        .await
        .expect("publish manifest");
    let mut commit_message = StreamMessage::commit_marker(&envelope, &marker).expect("commit");
    marker.transaction_id = " tx-partitioned".to_string();
    replace_header(
        &mut commit_message,
        "trellara.transaction_id",
        &marker.transaction_id,
    );
    commit_message.payload = marker.encode_to_vec().into();
    let commit_ack = publisher
        .publish(commit_message)
        .await
        .expect("publish commit");
    let mut partition_offsets = Vec::new();
    for chunk in &plan.chunks {
        let ack = publisher
            .publish(StreamMessage::partition_chunk(&envelope, chunk).expect("chunk"))
            .await
            .expect("publish chunk");
        partition_offsets.push(LocalPartitionChunkOffset {
            partition_id: chunk.partition_id,
            offset: ack.offset,
        });
    }

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets,
        },
    )
    .expect_err("invalid commit marker payload");

    assert!(matches!(
        error,
        LocalStreamError::BarrierProtocol(
            trellara_protocol::ProtocolError::InvalidCommitMarkerField {
                field: "transaction_id",
                ..
            }
        )
    ));

    fs::remove_dir_all(root).expect("cleanup");
}
