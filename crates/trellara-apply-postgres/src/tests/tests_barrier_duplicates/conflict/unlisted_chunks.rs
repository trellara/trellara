use super::*;

#[tokio::test]
async fn barrier_worker_rejects_unlisted_chunk_buffered_before_manifest() {
    let envelope = envelope("tx-unlisted-before-manifest", "0/16B7184");
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
    let listed_chunk = plan.chunks.first().expect("listed chunk");
    let unlisted_chunk_id = unlisted_partition_id(&plan.manifest);
    let unlisted_message = unlisted_chunk_message(&envelope, unlisted_chunk_id);
    let listed_message =
        StreamMessage::partition_chunk(&envelope, listed_chunk).expect("listed chunk message");
    let commit_message = StreamMessage::commit_marker(
        &envelope,
        &TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker"),
    )
    .expect("commit marker");
    let manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    let consumer = RecordingConsumer::new(vec![
        unlisted_message,
        listed_message,
        commit_message,
        manifest_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7184".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("unlisted chunk").is_some());
    assert!(worker.run_once().await.expect("listed chunk").is_some());
    assert!(worker.run_once().await.expect("commit marker").is_some());
    let error = worker.run_once().await.expect_err("unlisted chunk");

    assert_partition_not_in_manifest(error, unlisted_chunk_id);
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_unlisted_chunk_after_manifest() {
    let envelope = envelope("tx-unlisted-after-manifest", "0/16B7188");
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
    let listed_chunk = plan.chunks.first().expect("listed chunk");
    let listed_message =
        StreamMessage::partition_chunk(&envelope, listed_chunk).expect("listed chunk message");
    let manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    let unlisted_chunk_id = unlisted_partition_id(&plan.manifest);
    let unlisted_message = unlisted_chunk_message(&envelope, unlisted_chunk_id);
    let consumer = RecordingConsumer::new(vec![listed_message, manifest_message, unlisted_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7188".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("listed chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    let error = worker.run_once().await.expect_err("unlisted chunk");

    assert_partition_not_in_manifest(error, unlisted_chunk_id);
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}
