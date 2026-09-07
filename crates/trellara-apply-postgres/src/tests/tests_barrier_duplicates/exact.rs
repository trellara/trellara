use super::*;

#[tokio::test]
async fn barrier_worker_acks_exact_duplicate_chunks_after_apply() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-duplicate", "0/16B7150"));
    let duplicate_chunk_message = chunk_message.clone();
    let consumer = RecordingConsumer::new(vec![
        chunk_message,
        duplicate_chunk_message,
        manifest_message,
        commit_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7150".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(matches!(
        worker.run_once().await.expect("first chunk").expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));
    assert!(matches!(
        worker
            .run_once()
            .await
            .expect("duplicate chunk")
            .expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(matches!(
        worker.run_once().await.expect("manifest").expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));

    assert!(matches!(
        worker.run_once().await.expect("commit").expect("step"),
        BarrierApplyStep::Applied(_)
    ));

    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-duplicate".to_string()]
    );
    assert_eq!(consumer_probe.acked_keys().len(), 4);
}

#[tokio::test]
async fn barrier_worker_acks_exact_duplicate_manifests_after_apply() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-duplicate-manifest", "0/16B7160"));
    let duplicate_manifest_message = manifest_message.clone();
    let consumer = RecordingConsumer::new(vec![
        chunk_message,
        manifest_message,
        duplicate_manifest_message,
        commit_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7160".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    assert!(matches!(
        worker
            .run_once()
            .await
            .expect("duplicate manifest")
            .expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));
    assert!(consumer_probe.acked_keys().is_empty());

    assert!(matches!(
        worker.run_once().await.expect("commit").expect("step"),
        BarrierApplyStep::Applied(_)
    ));

    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-duplicate-manifest".to_string()]
    );
    assert_eq!(consumer_probe.acked_keys().len(), 4);
}

#[tokio::test]
async fn barrier_worker_acks_exact_duplicate_commit_markers_after_apply() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-duplicate-commit", "0/16B7170"));
    let duplicate_commit_message = commit_message.clone();
    let consumer = RecordingConsumer::new(vec![
        chunk_message,
        commit_message,
        duplicate_commit_message,
        manifest_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7170".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("commit").is_some());
    assert!(matches!(
        worker
            .run_once()
            .await
            .expect("duplicate commit")
            .expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));
    assert!(consumer_probe.acked_keys().is_empty());

    assert!(matches!(
        worker.run_once().await.expect("manifest").expect("step"),
        BarrierApplyStep::Applied(_)
    ));

    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-duplicate-commit".to_string()]
    );
    assert_eq!(consumer_probe.acked_keys().len(), 4);
}
