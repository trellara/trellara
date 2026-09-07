use super::*;

#[tokio::test]
async fn barrier_worker_reports_pending_barrier_transactions_after_idle() {
    let (chunk_message, manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-pending", "0/16B7130"));
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7130".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let stats = worker.run_until_idle().await.expect("stats");

    assert_eq!(stats.applied_transactions, 0);
    assert_eq!(
        stats.barrier_pending,
        BarrierPendingStats {
            transactions: 1,
            with_manifest: 1,
            with_commit_marker: 0,
            invalid_commit_marker: 0,
            missing_manifest: 0,
            missing_commit_marker: 1,
            complete_chunk_sets: 1,
            expected_chunks: 1,
            buffered_chunks: 1,
            missing_chunks: 0,
            extra_chunks: 0,
            buffered_messages: 2,
        }
    );
    assert_eq!(worker.pending_barrier_stats(), stats.barrier_pending);
    assert!(consumer_probe.acked_keys().is_empty());
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_reports_missing_manifest_after_idle() {
    let (chunk_message, _manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-needs-manifest", "0/16B7134"));
    let consumer = RecordingConsumer::new(vec![chunk_message, commit_message]);
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B7134".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let stats = worker.run_until_idle().await.expect("stats");

    assert_eq!(
        stats.barrier_pending,
        BarrierPendingStats {
            transactions: 1,
            with_manifest: 0,
            with_commit_marker: 1,
            invalid_commit_marker: 0,
            missing_manifest: 1,
            missing_commit_marker: 0,
            complete_chunk_sets: 0,
            expected_chunks: 0,
            buffered_chunks: 1,
            missing_chunks: 0,
            extra_chunks: 0,
            buffered_messages: 2,
        }
    );
    assert!(applier_probe.applied_transactions().is_empty());
}

#[tokio::test]
async fn barrier_worker_reports_missing_chunks_after_idle() {
    let transaction_id = "tx-missing-chunk";
    let commit_lsn = "0/16B7138";
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: commit_lsn.to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![
            ChangeRecord {
                transaction_id: transaction_id.to_string(),
                total_order: 1,
                table_order: 1,
                partition_order: 1,
                relation: Some(relation()),
                operation: Operation::Insert as i32,
                replica_identity: ReplicaIdentity::Default as i32,
                before: None,
                after: Some(row(vec![
                    ColumnValue::text("id", 23, "sale-1", true),
                    ColumnValue::text("store_id", 25, "store-1", false),
                    ColumnValue::text("amount_cents", 20, "1299", false),
                ])),
                idempotency_key: idempotency_key("source", commit_lsn, transaction_id, 1),
            },
            ChangeRecord {
                transaction_id: transaction_id.to_string(),
                total_order: 2,
                table_order: 2,
                partition_order: 1,
                relation: Some(relation()),
                operation: Operation::Insert as i32,
                replica_identity: ReplicaIdentity::Default as i32,
                before: None,
                after: Some(row(vec![
                    ColumnValue::text("id", 23, "sale-2", true),
                    ColumnValue::text("store_id", 25, "store-2", false),
                    ColumnValue::text("amount_cents", 20, "2199", false),
                ])),
                idempotency_key: idempotency_key("source", commit_lsn, transaction_id, 2),
            },
        ],
    });
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
    assert_eq!(plan.chunks.len(), 2);
    let chunk_message =
        StreamMessage::partition_chunk(&envelope, &plan.chunks[0]).expect("chunk message");
    let manifest_message =
        StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest");
    let commit_message = StreamMessage::commit_marker(
        &envelope,
        &TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker"),
    )
    .expect("commit marker");
    let consumer = RecordingConsumer::new(vec![chunk_message, manifest_message, commit_message]);
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 2,
        commit_lsn: commit_lsn.to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let stats = worker.run_until_idle().await.expect("stats");

    assert_eq!(
        stats.barrier_pending,
        BarrierPendingStats {
            transactions: 1,
            with_manifest: 1,
            with_commit_marker: 1,
            invalid_commit_marker: 0,
            missing_manifest: 0,
            missing_commit_marker: 0,
            complete_chunk_sets: 0,
            expected_chunks: 2,
            buffered_chunks: 1,
            missing_chunks: 1,
            extra_chunks: 0,
            buffered_messages: 3,
        }
    );
    assert!(applier_probe.applied_transactions().is_empty());
}
