use super::fixtures::{partitioned_envelope, partitioned_plan};
use super::*;
use std::fs;
use trellara_protocol::{BarrierHoldReason, TransactionCommitMarker};

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_missing_partition_chunk() {
    let root = temp_root("barrier-reconstruct-missing");
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
    let first_chunk = plan.chunks.first().expect("chunk");
    let chunk_ack = publisher
        .publish(StreamMessage::partition_chunk(&envelope, first_chunk).expect("chunk"))
        .await
        .expect("publish chunk");

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets: vec![LocalPartitionChunkOffset {
                partition_id: first_chunk.partition_id,
                offset: chunk_ack.offset,
            }],
        },
    )
    .expect_err("missing partition chunk");

    let missing_partitions = plan
        .manifest
        .partitions
        .iter()
        .filter_map(|partition| (partition.id != first_chunk.partition_id).then_some(partition.id))
        .collect::<Vec<_>>();
    assert!(matches!(
        error,
        LocalStreamError::BarrierTransactionHeld {
            transaction_id,
            reason: BarrierHoldReason::PartitionChunksMissing,
            missing_partitions: actual_missing,
        } if transaction_id == "tx-partitioned" && actual_missing == missing_partitions
    ));

    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_duplicate_partition_offsets() {
    let root = temp_root("barrier-reconstruct-duplicate-partition");
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
    let chunk_ack = publisher
        .publish(StreamMessage::partition_chunk(&envelope, chunk).expect("chunk"))
        .await
        .expect("publish chunk");

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets: vec![
                LocalPartitionChunkOffset {
                    partition_id: chunk.partition_id,
                    offset: chunk_ack.offset,
                },
                LocalPartitionChunkOffset {
                    partition_id: chunk.partition_id,
                    offset: chunk_ack.offset,
                },
            ],
        },
    )
    .expect_err("duplicate partition offset");

    assert!(matches!(
        error,
        LocalStreamError::DuplicateBarrierPartitionOffset { partition_id }
            if partition_id == chunk.partition_id
    ));

    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_invalid_request_identity_before_reads() {
    let root = temp_root("barrier-reconstruct-invalid-identity");

    for (field, source_id, dataset_id, expected_reason) in [
        ("source_id", "", "dataset", "must not be empty"),
        (
            "source_id",
            " source ",
            "dataset",
            "must not contain surrounding whitespace",
        ),
        ("dataset_id", "source", "", "must not be empty"),
        (
            "dataset_id",
            "source",
            " dataset ",
            "must not contain surrounding whitespace",
        ),
    ] {
        let error = reconstruct_local_barrier_transaction(
            &root,
            &LocalBarrierReconstructionRequest {
                source_id: source_id.to_string(),
                dataset_id: dataset_id.to_string(),
                manifest_offset: 0,
                commit_offset: 0,
                partition_offsets: Vec::new(),
            },
        )
        .expect_err("invalid request identity");

        assert!(matches!(
            error,
            LocalStreamError::InvalidBarrierReconstructionField { field: actual, reason }
                if actual == field && reason == expected_reason
        ));
    }

    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_negative_offsets_before_reads() {
    let root = temp_root("barrier-reconstruct-negative-offset");

    for (field, manifest_offset, commit_offset, partition_offsets) in [
        ("manifest_offset", -1, 0, Vec::new()),
        ("commit_offset", 0, -1, Vec::new()),
        (
            "partition_offsets[].offset",
            0,
            0,
            vec![LocalPartitionChunkOffset {
                partition_id: 0,
                offset: -1,
            }],
        ),
    ] {
        let error = reconstruct_local_barrier_transaction(
            &root,
            &LocalBarrierReconstructionRequest {
                source_id: "source".to_string(),
                dataset_id: "dataset".to_string(),
                manifest_offset,
                commit_offset,
                partition_offsets,
            },
        )
        .expect_err("negative offset");

        assert!(matches!(
            error,
            LocalStreamError::NegativeBarrierOffset { field: actual, offset: -1 }
                if actual == field
        ));
    }

    let _ = fs::remove_dir_all(root);
}
