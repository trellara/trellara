use super::*;

#[tokio::test]
async fn barrier_worker_applies_partitioned_transaction_from_local_stream_read_ahead() {
    let root = std::env::temp_dir().join(format!(
        "trellara-local-barrier-read-ahead-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let envelope = envelope("tx-local-barrier", "0/16B710C");
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
    let publisher = trellara_stream_local::LocalPublisher::new(
        trellara_stream_local::LocalPublisherConfig::new(&root),
    )
    .expect("publisher");
    publisher
        .publish(StreamMessage::transaction_manifest(&envelope, &plan.manifest).expect("manifest"))
        .await
        .expect("publish manifest");
    publisher
        .publish(
            StreamMessage::commit_marker(
                &envelope,
                &TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker"),
            )
            .expect("commit marker"),
        )
        .await
        .expect("publish commit");
    for chunk in &plan.chunks {
        publisher
            .publish(StreamMessage::partition_chunk(&envelope, chunk).expect("chunk"))
            .await
            .expect("publish chunk");
    }
    let topics = vec![
        "trellara.source.sales.manifest".to_string(),
        "trellara.source.sales.commit".to_string(),
        "trellara.source.sales.partition.0".to_string(),
        "trellara.source.sales.partition.1".to_string(),
        "trellara.source.sales.partition.2".to_string(),
        "trellara.source.sales.partition.3".to_string(),
        "trellara.source.sales.partition.4".to_string(),
        "trellara.source.sales.partition.5".to_string(),
        "trellara.source.sales.partition.6".to_string(),
        "trellara.source.sales.partition.7".to_string(),
    ];
    let consumer = trellara_stream_local::LocalConsumer::new(
        trellara_stream_local::LocalConsumerConfig::new(&root, "applier", topics),
    )
    .expect("consumer");
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B710C".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = BarrierAwareApplyWorker::new(consumer, applier);

    let stats = worker.run_until_idle().await.expect("apply stats");

    assert_eq!(stats.applied_transactions, 1);
    assert_eq!(stats.applied_changes, 1);
    assert_eq!(stats.barrier_pending.transactions, 0);
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-local-barrier"]
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}
