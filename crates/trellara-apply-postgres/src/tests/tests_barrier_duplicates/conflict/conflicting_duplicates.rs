use super::*;

#[tokio::test]
async fn barrier_worker_rejects_conflicting_duplicate_manifests() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-manifest-conflict", "0/16B7178"));
    let mut conflicting_manifest_message = manifest_message.clone();
    let mut conflicting_manifest =
        TransactionManifest::decode(conflicting_manifest_message.payload.as_ref())
            .expect("decode manifest");
    conflicting_manifest.affected_tables[0]
        .relation
        .as_mut()
        .expect("affected relation")
        .table = "sales_shadow".to_string();
    conflicting_manifest_message.payload = conflicting_manifest.encode_to_vec().into();

    let consumer = RecordingConsumer::new(vec![
        chunk_message,
        manifest_message,
        conflicting_manifest_message,
        commit_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7178".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    let error = worker.run_once().await.expect_err("conflicting manifest");

    assert!(matches!(
        error,
        ApplyWorkerError::DuplicateManifest { transaction_id }
            if transaction_id == "tx-manifest-conflict"
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_conflicting_duplicate_commit_markers_before_manifest() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-commit-conflict", "0/16B717C"));
    let mut conflicting_commit_message = commit_message.clone();
    let mut conflicting_marker =
        TransactionCommitMarker::decode(conflicting_commit_message.payload.as_ref())
            .expect("decode commit marker");
    conflicting_marker.global_event_count += 1;
    replace_header(
        &mut conflicting_commit_message,
        "trellara.global_event_count",
        &conflicting_marker.global_event_count.to_string(),
    );
    conflicting_commit_message.payload = conflicting_marker.encode_to_vec().into();

    let consumer = RecordingConsumer::new(vec![
        chunk_message,
        commit_message,
        conflicting_commit_message,
        manifest_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B717C".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("commit").is_some());
    let error = worker
        .run_once()
        .await
        .expect_err("conflicting duplicate commit marker");

    assert!(matches!(
        error,
        ApplyWorkerError::CommitMarkerMismatch { transaction_id }
            if transaction_id == "tx-commit-conflict"
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_conflicting_duplicate_chunks() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-conflict", "0/16B7180"));
    let mut conflicting_message = chunk_message.clone();
    let mut conflicting_chunk =
        PartitionChunk::decode(conflicting_message.payload.as_ref()).expect("decode chunk");
    conflicting_chunk.changes[0].idempotency_key =
        "source:0/16B7180:tx-conflict:1-conflict".to_string();
    conflicting_chunk.finalize_checksum();
    replace_header(
        &mut conflicting_message,
        "trellara.partition_checksum",
        &conflicting_chunk.checksum.to_string(),
    );
    conflicting_message.payload = conflicting_chunk.encode_to_vec().into();

    let consumer = RecordingConsumer::new(vec![
        chunk_message,
        conflicting_message,
        manifest_message,
        commit_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7180".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("first chunk").is_some());
    let error = worker.run_once().await.expect_err("conflicting duplicate");

    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::DuplicatePartitionChunk { .. })
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}
