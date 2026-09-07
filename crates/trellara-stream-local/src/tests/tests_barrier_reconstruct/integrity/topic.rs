use super::*;
use std::fs;
use trellara_protocol::TransactionCommitMarker;

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_partition_topic_mismatch() {
    let root = temp_root("barrier-reconstruct-partition-mismatch");
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
    let chunk = plan.chunks.first().expect("chunk");
    let wrong_partition_id = (0..4)
        .find(|partition_id| *partition_id != chunk.partition_id)
        .expect("different partition");
    let wrong_topic_message = StreamMessage {
        topic: format!("trellara.source_a.sales.partition.{wrong_partition_id}"),
        partition: Some(wrong_partition_id as i32),
        ..StreamMessage::partition_chunk(&envelope, chunk).expect("chunk")
    };
    let chunk_ack = publisher
        .publish(wrong_topic_message)
        .await
        .expect("publish mismatched chunk");

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets: vec![LocalPartitionChunkOffset {
                partition_id: wrong_partition_id,
                offset: chunk_ack.offset,
            }],
        },
    )
    .expect_err("partition mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierPartitionMismatch {
            expected_partition_id,
            actual_partition_id,
            ..
        } if expected_partition_id == wrong_partition_id
            && actual_partition_id == chunk.partition_id
    ));

    fs::remove_dir_all(root).expect("cleanup");
}
