use super::*;

#[tokio::test]
async fn apply_worker_acks_only_after_successful_apply() {
    let envelope = envelope("tx-1", "0/16B6C50");
    let message = StreamMessage::strict_transaction(&envelope).expect("message");
    let consumer = RecordingConsumer::new(vec![message]);
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6C50".to_string(),
    })]);
    let consumer_probe = consumer.clone();
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let step = worker
        .run_once()
        .await
        .expect("worker step")
        .expect("applied message");

    assert_eq!(
        step,
        ApplyStep {
            transaction_id: "tx-1".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            acked_messages: 1,
        }
    );
    assert_eq!(applier_probe.applied_transactions(), vec!["tx-1"]);
    assert_eq!(
        consumer_probe.acked_keys(),
        vec!["source:retail:sales:tx-1:0/16B6C50"]
    );
}

#[tokio::test]
async fn apply_worker_acks_duplicate_after_dedup_decision() {
    let envelope = envelope("tx-2", "0/16B6D28");
    let message = StreamMessage::strict_transaction(&envelope).expect("message");
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::SkippedDuplicate,
        applied_changes: 0,
        commit_lsn: "0/16B6D28".to_string(),
    })]);
    let mut worker = ApplyWorker::new(consumer, applier);

    let stats = worker.run_until_idle().await.expect("worker stats");

    assert_eq!(
        stats,
        ApplyRunStats {
            applied_transactions: 0,
            skipped_duplicates: 1,
            applied_changes: 0,
            acked_messages: 1,
            last_commit_lsn: Some("0/16B6D28".to_string()),
            barrier_pending: BarrierPendingStats::default(),
        }
    );
    assert_eq!(
        consumer_probe.acked_keys(),
        vec!["source:retail:sales:tx-2:0/16B6D28"]
    );
}

#[tokio::test]
async fn apply_worker_accepts_explicit_strict_transaction_kind() {
    let envelope = envelope("tx-explicit-strict-kind", "0/16B6FC0");
    let mut message = StreamMessage::strict_transaction(&envelope).expect("message");
    message.headers.push(StreamHeader::new(
        "trellara.message_kind",
        "strict_transaction",
    ));
    let consumer = RecordingConsumer::new(vec![message]);
    let consumer_probe = consumer.clone();
    let applier = RecordingApplier::new(vec![Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6FC0".to_string(),
    })]);
    let applier_probe = applier.clone();
    let mut worker = ApplyWorker::new(consumer, applier);

    let step = worker
        .run_once()
        .await
        .expect("strict step")
        .expect("applied");

    assert_eq!(step.transaction_id, "tx-explicit-strict-kind");
    assert_eq!(
        consumer_probe.acked_keys(),
        vec!["source:retail:sales:tx-explicit-strict-kind:0/16B6FC0"]
    );
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-explicit-strict-kind"]
    );
}
