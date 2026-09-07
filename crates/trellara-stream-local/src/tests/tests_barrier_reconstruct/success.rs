use super::fixtures::{partitioned_envelope, partitioned_plan};
use super::*;
use std::fs;
use trellara_protocol::TransactionCommitMarker;

#[tokio::test]
async fn reconstructs_committed_partitioned_transaction_from_local_offsets() {
    let root = temp_root("barrier-reconstruct");
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
        let ack = publisher
            .publish(StreamMessage::partition_chunk(&envelope, chunk).expect("chunk"))
            .await
            .expect("publish chunk");
        partition_offsets.push(LocalPartitionChunkOffset {
            partition_id: chunk.partition_id,
            offset: ack.offset,
        });
    }

    let reconstructed = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets,
        },
    )
    .expect("reconstruct");

    assert_eq!(reconstructed.manifest.transaction_id, "tx-partitioned");
    assert_eq!(reconstructed.commit_marker.transaction_id, "tx-partitioned");
    assert_eq!(reconstructed.chunks.len(), plan.chunks.len());
    let expected_orders = (1..=envelope.changes.len() as u32).collect::<Vec<_>>();
    assert_eq!(
        reconstructed
            .changes
            .iter()
            .map(|change| change.total_order)
            .collect::<Vec<_>>(),
        expected_orders
    );

    fs::remove_dir_all(root).expect("cleanup");
}
