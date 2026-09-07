use super::*;
use prost::Message;
use std::fs;
use trellara_protocol::TransactionCommitMarker;

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_manifest_header_payload_mismatch() {
    let root = temp_root("barrier-reconstruct-manifest-header-mismatch");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let mut manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    replace_header(&mut manifest_message, "trellara.global_event_count", "999");
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
    .expect_err("manifest header mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierHeaderPayloadMismatch {
            field: "global_event_count",
            header,
            payload,
            ..
        } if header == "999" && payload == plan.manifest.global_event_count.to_string()
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_missing_manifest_header() {
    let root = temp_root("barrier-reconstruct-missing-manifest-header");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");

    let mut manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    remove_header(&mut manifest_message, "trellara.transaction_id");
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
    .expect_err("missing manifest header");

    assert!(matches!(
        error,
        LocalStreamError::BarrierHeaderPayloadMismatch {
            field: "transaction_id",
            header,
            payload,
            ..
        } if header.is_empty() && payload == plan.manifest.transaction_id
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn reconstruct_local_barrier_transaction_rejects_manifest_affected_table_mismatch() {
    let root = temp_root("barrier-reconstruct-affected-table-mismatch");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let envelope = partitioned_envelope();
    let mut plan = partitioned_plan(&envelope);
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("marker");
    let mut manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    plan.manifest.affected_tables[0].event_count -= 1;
    manifest_message.payload = plan.manifest.encode_to_vec().into();

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
    .expect_err("manifest affected table mismatch");

    assert!(matches!(
        error,
        LocalStreamError::BarrierProtocol(
            trellara_protocol::ProtocolError::ManifestAffectedTableCountMismatch {
                transaction_id,
                ..
            }
        ) if transaction_id == "tx-partitioned"
    ));

    fs::remove_dir_all(root).expect("cleanup");
}
