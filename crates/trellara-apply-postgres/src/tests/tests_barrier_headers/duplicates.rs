use super::*;

#[test]
fn required_header_rejects_duplicate_values() {
    let headers = vec![
        StreamHeader::new("trellara.transaction_id", "tx-original"),
        StreamHeader::new("trellara.transaction_id", "tx-shadow"),
    ];

    let error = crate::barrier::required_header(&headers, "trellara.transaction_id")
        .expect_err("duplicate required header");

    assert!(matches!(
        error,
        ApplyWorkerError::InvalidHeaderField {
            key: "trellara.transaction_id",
            reason,
        } if reason == "must not appear more than once"
    ));
}

#[test]
fn optional_header_rejects_duplicate_values() {
    let headers = vec![
        StreamHeader::new("trellara.partition_id", "0"),
        StreamHeader::new("trellara.partition_id", "1"),
    ];

    let error = crate::barrier::optional_header(&headers, "trellara.partition_id")
        .expect_err("duplicate optional header");

    assert!(matches!(
        error,
        ApplyWorkerError::InvalidHeaderField {
            key: "trellara.partition_id",
            reason,
        } if reason == "must not appear more than once"
    ));
}

#[tokio::test]
async fn barrier_worker_rejects_duplicate_required_header_before_buffering() {
    let (mut chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-duplicate-header", "0/16B6FC1"));
    chunk_message.headers.push(StreamHeader::new(
        "trellara.transaction_id",
        "tx-shadow-header",
    ));
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FC1".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("duplicate header");

    assert!(matches!(
        error,
        ApplyWorkerError::InvalidHeaderField {
            key: "trellara.transaction_id",
            reason,
        } if reason == "must not appear more than once"
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_rejects_duplicate_optional_header_before_buffering() {
    let (mut chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-duplicate-optional-header", "0/16B6FC2"));
    chunk_message
        .headers
        .push(StreamHeader::new("trellara.partition_id", "1"));
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FC2".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("duplicate header");

    assert!(matches!(
        error,
        ApplyWorkerError::InvalidHeaderField {
            key: "trellara.partition_id",
            reason,
        } if reason == "must not appear more than once"
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}
