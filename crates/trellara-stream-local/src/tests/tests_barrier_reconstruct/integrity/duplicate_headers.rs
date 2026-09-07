use super::*;
use crate::LocalDurability;
use std::fs::{self, OpenOptions};
use trellara_protocol::TransactionCommitMarker;
use trellara_stream::StreamHeader;

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_duplicate_manifest_header() {
    let root = temp_root("barrier-reconstruct-duplicate-manifest-header");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let mut manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    manifest_message
        .headers
        .push(StreamHeader::new("trellara.transaction_id", "tx-shadow"));
    let manifest_offset = append_raw_message(&root, &manifest_message);
    let commit_ack = publisher
        .publish(StreamMessage::commit_marker(&envelope, &marker).expect("commit"))
        .await
        .expect("publish commit");

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset,
            commit_offset: commit_ack.offset,
            partition_offsets: Vec::new(),
        },
    )
    .expect_err("duplicate manifest header");

    assert!(matches!(
        error,
        LocalStreamError::DuplicateBarrierHeader {
            field: "transaction_id",
            ..
        }
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_duplicate_chunk_header() {
    let root = temp_root("barrier-reconstruct-duplicate-chunk-header");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");
    let chunk = plan.chunks.first().expect("chunk");

    let manifest_ack = publisher
        .publish(StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest"))
        .await
        .expect("publish manifest");
    let commit_ack = publisher
        .publish(StreamMessage::commit_marker(&envelope, &marker).expect("commit"))
        .await
        .expect("publish commit");
    let mut chunk_message = StreamMessage::partition_chunk(&envelope, chunk).expect("chunk");
    chunk_message
        .headers
        .push(StreamHeader::new("trellara.partition_id", "999"));
    let chunk_offset = append_raw_message(&root, &chunk_message);

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets: vec![LocalPartitionChunkOffset {
                partition_id: chunk.partition_id,
                offset: chunk_offset,
            }],
        },
    )
    .expect_err("duplicate chunk header");

    assert!(matches!(
        error,
        LocalStreamError::DuplicateBarrierHeader {
            field: "partition_id",
            ..
        }
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

fn append_raw_message(root: &std::path::Path, message: &StreamMessage) -> i64 {
    let path = crate::paths::topic_path(root, &message.topic);
    let index_path = crate::paths::topic_index_path(root, &message.topic);
    fs::create_dir_all(crate::paths::topic_dir(root)).expect("topic dir");
    let offset = if path.exists() {
        crate::index::recover_topic_index(&path, &index_path, LocalDurability::default())
            .expect("recover index")
            .positions
            .len() as i64
    } else {
        0
    };
    let append_position = path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("topic file");
    crate::frame::write_frame(&mut file, message).expect("raw frame");
    crate::index::write_index(&index_path, &[append_position], LocalDurability::default())
        .expect("index");
    offset
}
