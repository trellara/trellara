use super::*;

#[tokio::test]
async fn apply_worker_does_not_ack_when_apply_fails() {
    let envelope = envelope("tx-3", "0/16B6E00");
    let message = StreamMessage::strict_transaction(&envelope).expect("message");
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Err(ApplyError::UnsupportedOperation(999))]);
    let mut worker = ApplyWorker::new(consumer, applier);

    let result = worker.run_once().await;

    assert!(matches!(
        result,
        Err(ApplyWorkerError::Apply(ApplyError::UnsupportedOperation(
            999
        )))
    ));
    assert!(consumer_probe.acked_keys().is_empty());
}

#[tokio::test]
async fn apply_worker_rejects_strict_header_payload_transaction_mismatch() {
    let envelope = envelope("tx-strict-header-mismatch", "0/16B6E20");
    let mut message = StreamMessage::strict_transaction(&envelope).expect("message");
    replace_header(&mut message, "trellara.transaction_id", "tx-header-only");
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6E20".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("header mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::HeaderPayloadMismatch {
            field: "transaction_id",
            ..
        }
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn apply_worker_rejects_strict_header_payload_commit_mismatch() {
    let envelope = envelope("tx-strict-lsn-mismatch", "0/16B6E40");
    let mut message = StreamMessage::strict_transaction(&envelope).expect("message");
    replace_header(&mut message, "trellara.commit_lsn", "0/16B6E41");
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6E40".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("header mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::HeaderPayloadMismatch {
            field: "commit_lsn",
            ..
        }
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn apply_worker_rejects_strict_ddl_event_count_header_mismatch() {
    let mut envelope = envelope("tx-strict-ddl-count-mismatch", "0/16B6E60");
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-strict-ddl-count-mismatch",
        2,
        relation(),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();
    let mut message = StreamMessage::strict_transaction(&envelope).expect("message");
    replace_header(&mut message, "trellara.ddl_event_count", "0");
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6E60".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("header mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::HeaderPayloadMismatch {
            field: "ddl_event_count",
            ..
        }
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn apply_worker_rejects_strict_partitioned_scale_decision_header_mismatch() {
    let envelope = envelope("tx-strict-replay-decision-mismatch", "0/16B6E70");
    let mut message = StreamMessage::strict_transaction(&envelope).expect("message");
    replace_header(
        &mut message,
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required",
    );
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6E70".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("header mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::HeaderPayloadMismatch {
            field: "partitioned_scale_decision",
            ..
        }
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn apply_worker_does_not_ack_malformed_envelopes() {
    let mut message =
        StreamMessage::strict_transaction(&envelope("tx-4", "0/16B6F80")).expect("message");
    message.payload = vec![0, 1, 2, 3].into();
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6F80".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let result = worker.run_once().await;

    assert!(matches!(result, Err(ApplyWorkerError::Protocol(_))));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn apply_worker_rejects_barrier_messages_without_apply_or_ack() {
    let (chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-strict-rejects-chunk", "0/16B6FA0"));
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FA0".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let result = worker.run_once().await;

    assert!(matches!(
        result,
        Err(ApplyWorkerError::UnsupportedMessageKind(kind)) if kind == "partition_chunk"
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn apply_worker_does_not_ack_when_apply_outcome_commit_lsn_mismatches_envelope() {
    let envelope = envelope("tx-outcome-lsn-mismatch", "0/16B6FC0");
    let message = StreamMessage::strict_transaction(&envelope).expect("message");
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FC1".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("outcome mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::ApplyOutcomeCommitLsnMismatch {
            transaction_id,
            expected,
            actual,
        } if transaction_id == "tx-outcome-lsn-mismatch"
            && expected == "0/16B6FC0"
            && actual == "0/16B6FC1"
    ));
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-outcome-lsn-mismatch"]
    );
    assert!(consumer_probe.acked_keys().is_empty());
}

#[tokio::test]
async fn apply_worker_does_not_ack_when_duplicate_outcome_commit_lsn_mismatches_envelope() {
    let envelope = envelope("tx-duplicate-outcome-lsn-mismatch", "0/16B6FD0");
    let message = StreamMessage::strict_transaction(&envelope).expect("message");
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::SkippedDuplicate,
        applied_changes: 0,
        commit_lsn: "0/16B6FD1".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("outcome mismatch");

    assert!(matches!(
        error,
        ApplyWorkerError::ApplyOutcomeCommitLsnMismatch {
            transaction_id,
            expected,
            actual,
        } if transaction_id == "tx-duplicate-outcome-lsn-mismatch"
            && expected == "0/16B6FD0"
            && actual == "0/16B6FD1"
    ));
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-duplicate-outcome-lsn-mismatch"]
    );
    assert!(consumer_probe.acked_keys().is_empty());
}
