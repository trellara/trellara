use super::*;

#[tokio::test]
async fn barrier_worker_rejects_padded_chunk_routing_headers_before_buffering() {
    for header in [
        "trellara.source_id",
        "trellara.dataset_id",
        "trellara.database_id",
        "trellara.transaction_id",
        "trellara.commit_lsn",
        "trellara.partitioned_scale_decision",
        "trellara.partition_parallel_safe",
        "trellara.requires_ddl_barrier",
        "trellara.dml_replay_after_ddl_barrier_required",
        "trellara.partitioned_scale_reason",
        "trellara.partition_id",
        "trellara.partition_event_count",
        "trellara.partition_checksum",
    ] {
        let (mut chunk_message, _manifest_message, _commit_message) =
            partitioned_messages(&envelope("tx-padded-header", "0/16B6FA7"));
        let original_value = chunk_message
            .headers
            .iter()
            .find(|candidate| candidate.key == header)
            .expect("header")
            .value
            .clone();
        replace_header(&mut chunk_message, header, &format!(" {original_value}"));
        let consumer = RecordingConsumer::new(vec![chunk_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FA7".to_string(),
        })]);
        let applier_probe = applier.clone();
        let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

        let error = worker.run_once().await.expect_err("padded header");

        assert!(matches!(
            error,
            ApplyWorkerError::InvalidHeaderField {
                key,
                reason,
            } if key == header && reason == "must not contain surrounding whitespace"
        ));
        assert!(consumer_probe.acked_keys().is_empty());
        assert!(applier_probe.applied_transactions().is_empty());
    }
}

#[tokio::test]
async fn barrier_worker_rejects_chunk_header_payload_transaction_mismatch() {
    let (mut chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-chunk-header-mismatch", "0/16B6FA8"));
    replace_header(
        &mut chunk_message,
        "trellara.transaction_id",
        "tx-header-only",
    );
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FA8".to_string(),
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
async fn barrier_worker_rejects_chunk_partition_routing_header_mismatch() {
    for (header, value, field) in [
        ("trellara.partition_id", "999", "partition_id"),
        (
            "trellara.partition_event_count",
            "999",
            "partition_event_count",
        ),
        ("trellara.partition_checksum", "999", "partition_checksum"),
    ] {
        let (mut chunk_message, _manifest_message, _commit_message) =
            partitioned_messages(&envelope("tx-chunk-routing-mismatch", "0/16B6FAC"));
        replace_header(&mut chunk_message, header, value);
        let consumer = RecordingConsumer::new(vec![chunk_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FAC".to_string(),
        })]);
        let applier_probe = applier.clone();
        let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

        let error = worker.run_once().await.expect_err("header mismatch");

        assert!(matches!(
            error,
            ApplyWorkerError::HeaderPayloadMismatch {
                field: actual,
                ..
            } if actual == field
        ));
        assert!(consumer_probe.acked_keys().is_empty());
        assert!(applier_probe.applied_transactions().is_empty());
    }
}

#[tokio::test]
async fn barrier_worker_rejects_chunk_partitioned_scale_readiness_header_mismatch() {
    for (header, value, field) in [
        (
            "trellara.partitioned_scale_decision",
            "ddl_barrier_required",
            "partitioned_scale_decision",
        ),
        (
            "trellara.partition_parallel_safe",
            "false",
            "partition_parallel_safe",
        ),
        (
            "trellara.requires_ddl_barrier",
            "true",
            "requires_ddl_barrier",
        ),
        (
            "trellara.dml_replay_after_ddl_barrier_required",
            "true",
            "dml_replay_after_ddl_barrier_required",
        ),
        (
            "trellara.partitioned_scale_reason",
            "mixed DDL and DML transaction must apply DDL barrier before partitioned DML replay",
            "partitioned_scale_reason",
        ),
    ] {
        let (mut chunk_message, _manifest_message, _commit_message) =
            partitioned_messages(&envelope("tx-chunk-replay-decision-mismatch", "0/16B6FAE"));
        replace_header(&mut chunk_message, header, value);
        let consumer = RecordingConsumer::new(vec![chunk_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FAE".to_string(),
        })]);
        let applier_probe = applier.clone();
        let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

        let error = worker.run_once().await.expect_err("header mismatch");

        assert!(matches!(
            error,
            ApplyWorkerError::HeaderPayloadMismatch { field: actual, .. } if actual == field
        ));
        assert!(consumer_probe.acked_keys().is_empty());
        assert!(applier_probe.applied_transactions().is_empty());
    }
}

#[tokio::test]
async fn barrier_worker_requires_partitioned_scale_readiness_headers() {
    for header in [
        "trellara.partitioned_scale_decision",
        "trellara.partition_parallel_safe",
        "trellara.requires_ddl_barrier",
        "trellara.dml_replay_after_ddl_barrier_required",
        "trellara.partitioned_scale_reason",
    ] {
        let (mut chunk_message, _manifest_message, _commit_message) =
            partitioned_messages(&envelope("tx-chunk-replay-decision-missing", "0/16B6FAF"));
        remove_header(&mut chunk_message, header);
        let consumer = RecordingConsumer::new(vec![chunk_message]);
        let consumer_probe = consumer.clone();
        let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6FAF".to_string(),
        })]);
        let applier_probe = applier.clone();
        let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

        let error = worker.run_once().await.expect_err("missing header");

        assert!(matches!(
            error,
            ApplyWorkerError::MissingHeader(actual) if actual == header
        ));
        assert!(consumer_probe.acked_keys().is_empty());
        assert!(applier_probe.applied_transactions().is_empty());
    }
}

#[tokio::test]
async fn barrier_worker_rejects_legacy_chunk_partitioned_scale_decision_header_mismatch() {
    let (mut chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-chunk-replay-decision-mismatch", "0/16B6FAE"));
    replace_header(
        &mut chunk_message,
        "trellara.partitioned_scale_decision",
        "ddl_barrier_required",
    );
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FAE".to_string(),
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
async fn barrier_worker_requires_legacy_partitioned_scale_decision_header() {
    let (mut chunk_message, _manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-chunk-replay-decision-missing", "0/16B6FAF"));
    remove_header(&mut chunk_message, "trellara.partitioned_scale_decision");
    let consumer = RecordingConsumer::new(vec![chunk_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FAF".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let error = worker.run_once().await.expect_err("missing header");

    assert!(matches!(
        error,
        ApplyWorkerError::MissingHeader("trellara.partitioned_scale_decision")
    ));
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}
