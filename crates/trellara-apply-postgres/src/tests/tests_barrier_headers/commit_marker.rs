use super::*;

#[tokio::test]
async fn barrier_worker_rejects_commit_marker_header_payload_transaction_mismatch() {
    let (_chunk_message, _manifest_message, mut commit_message) =
        partitioned_messages(&envelope("tx-marker-header-mismatch", "0/16B6FB8"));
    replace_header(
        &mut commit_message,
        "trellara.transaction_id",
        "tx-header-only",
    );
    let consumer = RecordingConsumer::new(vec![commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FB8".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

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
async fn barrier_worker_rejects_commit_marker_partitioned_scale_decision_header_mismatch() {
    let (_chunk_message, _manifest_message, mut commit_message) =
        partitioned_messages(&envelope("tx-marker-replay-decision-mismatch", "0/16B6FBC"));
    replace_header(
        &mut commit_message,
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required",
    );
    let consumer = RecordingConsumer::new(vec![commit_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FBC".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

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
async fn barrier_worker_rejects_commit_marker_proof_header_mismatch() {
    for (header, value, expected_field) in [
        ("trellara.global_event_count", "999", "global_event_count"),
        ("trellara.partition_count", "999", "partition_count"),
        ("trellara.manifest_checksum", "999", "manifest_checksum"),
    ] {
        let (_chunk_message, _manifest_message, mut commit_message) =
            partitioned_messages(&envelope("tx-marker-proof-header-mismatch", "0/16B6FC0"));
        replace_header(&mut commit_message, header, value);
        let consumer = RecordingConsumer::new(vec![commit_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FC0".to_string(),
        })]);
        let applier_probe = applier.clone();
        let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

        let error = worker.run_once().await.expect_err("header mismatch");

        assert!(matches!(
            error,
            ApplyWorkerError::HeaderPayloadMismatch { field, .. } if field == expected_field
        ));
        assert!(consumer_probe.acked_keys().is_empty());
        assert!(applier_probe.applied_transactions().is_empty());
    }
}
