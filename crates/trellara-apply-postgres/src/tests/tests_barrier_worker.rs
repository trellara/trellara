use super::*;

#[tokio::test]
async fn barrier_worker_rejects_mismatched_commit_marker() {
    let (chunk_message, manifest_message, mut commit_message) =
        partitioned_messages(&envelope("tx-marker", "0/16B7190"));
    let mut marker =
        TransactionCommitMarker::decode(commit_message.payload.as_ref()).expect("decode marker");
    marker.global_event_count += 1;
    replace_header(
        &mut commit_message,
        "trellara.global_event_count",
        &marker.global_event_count.to_string(),
    );
    commit_message.payload = marker.encode_to_vec().into();
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7190".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    let error = worker
        .run_once()
        .await
        .expect_err("mismatched commit marker");

    assert!(matches!(
        error,
        ApplyWorkerError::CommitMarkerMismatch { transaction_id }
            if transaction_id == "tx-marker"
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_invalid_commit_marker_before_apply_or_ack() {
    let (chunk_message, manifest_message, mut commit_message) =
        partitioned_messages(&envelope("tx-invalid-marker", "0/16B7193"));
    let mut marker =
        TransactionCommitMarker::decode(commit_message.payload.as_ref()).expect("decode marker");
    marker.transaction_id = " tx-invalid-marker".to_string();
    commit_message.payload = marker.encode_to_vec().into();
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7193".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    let error = worker.run_once().await.expect_err("invalid commit marker");

    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::InvalidCommitMarkerField {
            field: "transaction_id",
            ..
        })
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_duplicate_manifest_partitions_before_apply() {
    let envelope = envelope("tx-duplicate-manifest", "0/16B7191");
    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    let chunk = plan.chunks.first().expect("chunk");
    let chunk_message = StreamMessage::partition_chunk(&envelope, chunk).expect("chunk message");
    let commit_message = StreamMessage::commit_marker(
        &envelope,
        &TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker"),
    )
    .expect("commit marker message");
    let mut duplicate_manifest = plan.manifest.clone();
    duplicate_manifest
        .partitions
        .push(duplicate_manifest.partitions[0].clone());
    let mut manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    replace_header(
        &mut manifest_message,
        "trellara.partition_count",
        &duplicate_manifest.partitions.len().to_string(),
    );
    manifest_message.payload = duplicate_manifest.encode_to_vec().into();
    let consumer = RecordingConsumer::new(vec![chunk_message, commit_message, manifest_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7191".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("marker").is_some());
    let error = worker
        .run_once()
        .await
        .expect_err("duplicate manifest partition");

    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::DuplicateManifestPartition {
            transaction_id,
            ..
        }) if transaction_id == "tx-duplicate-manifest"
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_corrupt_chunk_before_apply_or_ack() {
    let (mut chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-corrupt-chunk", "0/16B7192"));
    let mut chunk = PartitionChunk::decode(chunk_message.payload.as_ref()).expect("decode chunk");
    chunk.checksum += 1;
    replace_header(
        &mut chunk_message,
        "trellara.partition_checksum",
        &chunk.checksum.to_string(),
    );
    chunk_message.payload = chunk.encode_to_vec().into();
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7192".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("corrupt chunk");

    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::PartitionChecksumMismatch { .. })
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_invalid_chunk_identity_before_buffering() {
    let (mut chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-invalid-chunk", "0/16B7194"));
    let mut chunk = PartitionChunk::decode(chunk_message.payload.as_ref()).expect("decode chunk");
    chunk.transaction_id = " tx-invalid-chunk".to_string();
    chunk.finalize_checksum();
    chunk_message.payload = chunk.encode_to_vec().into();
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7194".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("invalid chunk");

    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::InvalidPartitionChunkField {
            field: "transaction_id",
            ..
        })
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_does_not_ack_when_apply_fails() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-7", "0/16B7200"));
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Err(ApplyError::UnsupportedOperation(999))]);
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("buffer").is_some());
    assert!(worker.run_once().await.expect("buffer").is_some());
    let result = worker.run_once().await;

    assert!(matches!(
        result,
        Err(ApplyWorkerError::Apply(ApplyError::UnsupportedOperation(
            999
        )))
    ));
    assert!(consumer_probe.acked_keys().is_empty());
}
