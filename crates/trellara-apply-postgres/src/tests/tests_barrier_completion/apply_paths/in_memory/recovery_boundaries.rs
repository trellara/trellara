use super::*;

#[tokio::test]
async fn barrier_worker_keeps_ready_transaction_pending_when_ack_fails_after_apply() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-barrier-ack-fail", "0/16B7108"));
    let chunk_key = chunk_message.key.clone();
    let consumer = RecordingConsumer::failing_second_ack(vec![
        chunk_message,
        manifest_message,
        commit_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7108".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    let error = worker.run_once().await.expect_err("ack failure");

    assert!(matches!(
        error,
        ApplyWorkerError::Stream(StreamError::Consumer(message))
            if message == "simulated ack failure after apply"
    ));
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-barrier-ack-fail"]
    );
    assert_eq!(consumer_probe.acked_keys(), vec![chunk_key]);
    assert_eq!(
        worker.pending_barrier_stats(),
        BarrierPendingStats {
            transactions: 1,
            with_manifest: 1,
            with_commit_marker: 1,
            invalid_commit_marker: 0,
            missing_manifest: 0,
            missing_commit_marker: 0,
            complete_chunk_sets: 1,
            expected_chunks: 1,
            buffered_chunks: 1,
            missing_chunks: 0,
            extra_chunks: 0,
            buffered_messages: 3,
        }
    );
}

#[tokio::test]
async fn barrier_worker_retries_ready_pending_transaction_after_ack_failure() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-barrier-ack-retry", "0/16B7110"));
    let chunk_key = chunk_message.key.clone();
    let consumer = RecordingConsumer::failing_second_ack(vec![
        chunk_message,
        manifest_message,
        commit_message,
    ]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![
        Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B7110".to_string(),
        }),
        Ok(ApplyOutcome {
            decision: ApplyDecision::SkippedDuplicate,
            applied_changes: 0,
            commit_lsn: "0/16B7110".to_string(),
        }),
    ]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    assert!(worker
        .run_once()
        .await
        .expect_err("ack failure")
        .to_string()
        .contains("simulated ack failure"));

    let retried = worker
        .run_once()
        .await
        .expect("retry ready pending")
        .expect("retry step");

    assert!(matches!(
        retried,
        BarrierApplyStep::Applied(ApplyStep {
            transaction_id,
            decision: ApplyDecision::SkippedDuplicate,
            ..
        }) if transaction_id == "tx-barrier-ack-retry"
    ));
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-barrier-ack-retry", "tx-barrier-ack-retry"]
    );
    assert_eq!(
        consumer_probe.acked_keys(),
        vec![
            chunk_key.clone(),
            chunk_key,
            "source:retail:sales:tx-barrier-ack-retry:0/16B7110".to_string(),
            "source:retail:sales:tx-barrier-ack-retry:0/16B7110".to_string(),
        ]
    );
    assert_eq!(
        worker.pending_barrier_stats(),
        BarrierPendingStats::default()
    );
}

#[tokio::test]
async fn barrier_worker_waits_for_commit_marker_before_apply() {
    let (chunk_message, manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-needs-commit", "0/16B7120"));
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7120".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(matches!(
        worker.run_once().await.expect("manifest").expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));
    assert!(worker.run_once().await.expect("idle").is_none());

    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}
