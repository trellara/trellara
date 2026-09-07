use super::*;
use std::fs;
use trellara_protocol::TransactionCommitMarker;

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_manifest_replay_decision_mismatch() {
    let root = temp_root("barrier-reconstruct-manifest-replay-decision-mismatch");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let mut manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    replace_header(
        &mut manifest_message,
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required",
    );
    let manifest_ack = publisher
        .publish(manifest_message)
        .await
        .expect("publish manifest");
    let commit_ack = publisher
        .publish(StreamMessage::commit_marker(&envelope, &marker).expect("commit"))
        .await
        .expect("publish commit");

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets: Vec::new(),
        },
    )
    .expect_err("manifest replay decision mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierHeaderPayloadMismatch {
            field: "partitioned_scale_decision",
            header,
            payload,
            ..
        } if header == "ddl_barrier_required" && payload == "partition_parallel_dml"
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_commit_header_payload_mismatch() {
    let root = temp_root("barrier-reconstruct-commit-header-mismatch");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let manifest_ack = publisher
        .publish(StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest"))
        .await
        .expect("publish manifest");
    let mut commit_message = StreamMessage::commit_marker(&envelope, &marker).expect("commit");
    replace_header(&mut commit_message, "trellara.manifest_checksum", "999");
    let commit_ack = publisher
        .publish(commit_message)
        .await
        .expect("publish commit");

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets: Vec::new(),
        },
    )
    .expect_err("commit header mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierHeaderPayloadMismatch {
            field: "manifest_checksum",
            header,
            payload,
            ..
        } if header == "999" && payload == marker.manifest_checksum.to_string()
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_commit_replay_decision_mismatch() {
    let root = temp_root("barrier-reconstruct-commit-replay-decision-mismatch");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let manifest_ack = publisher
        .publish(StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest"))
        .await
        .expect("publish manifest");
    let mut commit_message = StreamMessage::commit_marker(&envelope, &marker).expect("commit");
    replace_header(
        &mut commit_message,
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required",
    );
    let commit_ack = publisher
        .publish(commit_message)
        .await
        .expect("publish commit");

    let error = reconstruct_local_barrier_transaction(
        &root,
        &LocalBarrierReconstructionRequest {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            manifest_offset: manifest_ack.offset,
            commit_offset: commit_ack.offset,
            partition_offsets: Vec::new(),
        },
    )
    .expect_err("commit replay decision mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierHeaderPayloadMismatch {
            field: "partitioned_scale_decision",
            header,
            payload,
            ..
        } if header == "ddl_barrier_required" && payload == "partition_parallel_dml"
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_chunk_header_payload_mismatch() {
    let root = temp_root("barrier-reconstruct-header-mismatch");
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
    let mut chunk_message = StreamMessage::partition_chunk(&envelope, chunk).expect("chunk");
    replace_header(&mut chunk_message, "trellara.partition_checksum", "999");
    let chunk_ack = publisher
        .publish(chunk_message)
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
                partition_id: chunk.partition_id,
                offset: chunk_ack.offset,
            }],
        },
    )
    .expect_err("header mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierHeaderPayloadMismatch {
            field: "partition_checksum",
            header,
            payload,
            ..
        } if header == "999" && payload == chunk.checksum.to_string()
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_chunk_replay_decision_mismatch() {
    let root = temp_root("barrier-reconstruct-chunk-replay-decision-mismatch");
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
    let mut chunk_message = StreamMessage::partition_chunk(&envelope, chunk).expect("chunk");
    replace_header(
        &mut chunk_message,
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required",
    );
    let chunk_ack = publisher
        .publish(chunk_message)
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
                partition_id: chunk.partition_id,
                offset: chunk_ack.offset,
            }],
        },
    )
    .expect_err("chunk replay decision mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierHeaderPayloadMismatch {
            field: "partitioned_scale_decision",
            header,
            payload,
            ..
        } if header == "ddl_barrier_required" && payload == "partition_parallel_dml"
    ));

    fs::remove_dir_all(root).expect("cleanup");
}
