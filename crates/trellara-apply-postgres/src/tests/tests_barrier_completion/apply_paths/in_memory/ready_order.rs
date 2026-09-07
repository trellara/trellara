use super::*;

#[tokio::test]
async fn barrier_worker_waits_for_manifest_before_apply_and_ack() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-5", "0/16B7000"));
    let chunk_key = chunk_message.key.clone();
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7000".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let buffered = worker
        .run_once()
        .await
        .expect("buffer step")
        .expect("buffered message");
    assert!(matches!(buffered, BarrierApplyStep::Buffered { .. }));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());

    let buffered = worker
        .run_once()
        .await
        .expect("manifest buffer")
        .expect("buffered manifest");
    assert!(matches!(buffered, BarrierApplyStep::Buffered { .. }));
    assert!(consumer_probe.acked_keys().is_empty());

    let applied = worker
        .run_once()
        .await
        .expect("apply step")
        .expect("applied transaction");
    assert!(
        matches!(applied, BarrierApplyStep::Applied(ApplyStep { transaction_id, .. }) if transaction_id == "tx-5")
    );
    assert_eq!(applier_probe.applied_transactions(), vec!["tx-5"]);
    assert_eq!(applier_probe.applied_manifest_partition_counts(), vec![1]);
    assert_eq!(consumer_probe.acked_keys().len(), 3);
    assert_eq!(
        consumer_probe.acked_keys(),
        vec![
            chunk_key,
            "source:retail:sales:tx-5:0/16B7000".to_string(),
            "source:retail:sales:tx-5:0/16B7000".to_string(),
        ]
    );
    assert!(consumer_probe
        .acked_keys()
        .last()
        .expect("commit ack")
        .contains("tx-5"));
}

#[tokio::test]
async fn barrier_worker_does_not_ack_when_apply_outcome_commit_lsn_mismatches_envelope() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-barrier-outcome-lsn-mismatch", "0/16B7008"));
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7009".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    let error = worker.run_once().await.expect_err("outcome mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::ApplyOutcomeCommitLsnMismatch {
            transaction_id,
            expected,
            actual,
        } if transaction_id == "tx-barrier-outcome-lsn-mismatch"
            && expected == "0/16B7008"
            && actual == "0/16B7009"
    ));
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-barrier-outcome-lsn-mismatch"]
    );
    assert!(consumer_probe.acked_keys().is_empty());
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
async fn barrier_worker_does_not_ack_when_duplicate_outcome_commit_lsn_mismatches_envelope() {
    let (chunk_message, manifest_message, commit_message) = partitioned_messages(&envelope(
        "tx-barrier-duplicate-outcome-lsn-mismatch",
        "0/16B7010",
    ));
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::SkippedDuplicate,
        applied_changes: 0,
        commit_lsn: "0/16B7011".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    let error = worker.run_once().await.expect_err("outcome mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::ApplyOutcomeCommitLsnMismatch {
            transaction_id,
            expected,
            actual,
        } if transaction_id == "tx-barrier-duplicate-outcome-lsn-mismatch"
            && expected == "0/16B7010"
            && actual == "0/16B7011"
    ));
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-barrier-duplicate-outcome-lsn-mismatch"]
    );
    assert!(consumer_probe.acked_keys().is_empty());
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
async fn barrier_worker_accepts_manifest_before_chunks() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-6", "0/16B7100"));
    let consumer = RecordingConsumer::new(vec![manifest_message, commit_message, chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7100".to_string(),
    })]);
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(matches!(
        worker.run_once().await.expect("buffer").expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));
    assert!(matches!(
        worker.run_once().await.expect("commit").expect("step"),
        BarrierApplyStep::Buffered { .. }
    ));
    assert!(consumer_probe.acked_keys().is_empty());

    assert!(matches!(
        worker.run_once().await.expect("apply").expect("step"),
        BarrierApplyStep::Applied(_)
    ));
    assert_eq!(consumer_probe.acked_keys().len(), 3);
}

#[tokio::test]
async fn barrier_worker_applies_strict_chunk_barrier_messages() {
    let (chunk_message, manifest_message, commit_message) =
        strict_chunked_messages(&envelope("tx-strict-chunk-barrier", "0/16B7110"));
    assert!(chunk_message
        .headers
        .contains(&StreamHeader::new("trellara.message_kind", "strict_chunk")));
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7110".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    assert!(worker.run_once().await.expect("strict chunk").is_some());
    assert!(worker.run_once().await.expect("manifest").is_some());
    assert!(matches!(
        worker.run_once().await.expect("commit").expect("step"),
        BarrierApplyStep::Applied(_)
    ));
    assert_eq!(
        consumer_probe.acked_keys(),
        vec![
            "source:retail:sales:tx-strict-chunk-barrier:0/16B7110:0",
            "source:retail:sales:tx-strict-chunk-barrier:0/16B7110",
            "source:retail:sales:tx-strict-chunk-barrier:0/16B7110",
        ]
    );
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-strict-chunk-barrier"]
    );
}
