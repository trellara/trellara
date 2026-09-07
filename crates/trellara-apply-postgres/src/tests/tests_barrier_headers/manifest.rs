use super::*;

#[tokio::test]
async fn barrier_worker_rejects_manifest_header_payload_commit_mismatch() {
    let (_chunk_message, mut manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-manifest-header-mismatch", "0/16B6FB0"));
    replace_header(&mut manifest_message, "trellara.commit_lsn", "0/16B6FB1");
    let consumer = RecordingConsumer::new(vec![manifest_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FB0".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

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
async fn barrier_worker_rejects_manifest_partitioned_scale_decision_header_mismatch() {
    let (_chunk_message, mut manifest_message, _commit_message) = partitioned_messages(&envelope(
        "tx-manifest-replay-decision-mismatch",
        "0/16B6FB4",
    ));
    replace_header(
        &mut manifest_message,
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required",
    );
    let consumer = RecordingConsumer::new(vec![manifest_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FB4".to_string(),
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
async fn barrier_worker_rejects_manifest_count_header_mismatch() {
    for (header, value, expected_field) in [
        ("trellara.global_event_count", "999", "global_event_count"),
        ("trellara.partition_count", "999", "partition_count"),
    ] {
        let (_chunk_message, mut manifest_message, _commit_message) =
            partitioned_messages(&envelope("tx-manifest-count-header-mismatch", "0/16B6FC4"));
        replace_header(&mut manifest_message, header, value);
        let consumer = RecordingConsumer::new(vec![manifest_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FC4".to_string(),
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

#[tokio::test]
async fn barrier_worker_rejects_manifest_evidence_header_mismatch() {
    for (header, value, expected_field) in [
        (
            "trellara.partitioned_scale_manifest_checksum",
            "999",
            "partitioned_scale_manifest_checksum",
        ),
        (
            "trellara.partitioned_scale_manifest_event_count",
            "999",
            "partitioned_scale_manifest_event_count",
        ),
        (
            "trellara.partitioned_scale_envelope_event_count",
            "999",
            "partitioned_scale_envelope_event_count",
        ),
        (
            "trellara.partitioned_scale_event_count_coverage",
            "false",
            "partitioned_scale_event_count_coverage",
        ),
        (
            "trellara.partitioned_scale_participating_partition_ids",
            "999",
            "partitioned_scale_participating_partition_ids",
        ),
        (
            "trellara.partitioned_scale_visibility_contract",
            "partition-local visibility is enough",
            "partitioned_scale_visibility_contract",
        ),
    ] {
        let (_chunk_message, mut manifest_message, _commit_message) =
            partitioned_messages(&envelope("tx-manifest-evidence-mismatch", "0/16B6FC5"));
        replace_header(&mut manifest_message, header, value);
        let consumer = RecordingConsumer::new(vec![manifest_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FC5".to_string(),
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

#[tokio::test]
async fn barrier_worker_rejects_manifest_missing_evidence_header() {
    for (header, expected_field) in [
        (
            "trellara.partitioned_scale_manifest_checksum",
            "partitioned_scale_manifest_checksum",
        ),
        (
            "trellara.partitioned_scale_manifest_event_count",
            "partitioned_scale_manifest_event_count",
        ),
        (
            "trellara.partitioned_scale_envelope_event_count",
            "partitioned_scale_envelope_event_count",
        ),
        (
            "trellara.partitioned_scale_event_count_coverage",
            "partitioned_scale_event_count_coverage",
        ),
        (
            "trellara.partitioned_scale_participating_partition_ids",
            "partitioned_scale_participating_partition_ids",
        ),
        (
            "trellara.partitioned_scale_visibility_contract",
            "partitioned_scale_visibility_contract",
        ),
    ] {
        let (_chunk_message, mut manifest_message, _commit_message) =
            partitioned_messages(&envelope("tx-manifest-evidence-missing", "0/16B6FC6"));
        remove_header(&mut manifest_message, header);
        let consumer = RecordingConsumer::new(vec![manifest_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FC6".to_string(),
        })]);
        let applier_probe = applier.clone();
        let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

        let error = worker.run_once().await.expect_err("missing evidence");

        assert!(matches!(
            error,
            ApplyWorkerError::HeaderPayloadMismatch { field, .. } if field == expected_field
        ));
        assert!(consumer_probe.acked_keys().is_empty());
        assert!(applier_probe.applied_transactions().is_empty());
    }
}
