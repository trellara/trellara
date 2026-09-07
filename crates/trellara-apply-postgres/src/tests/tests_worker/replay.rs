use super::*;

#[tokio::test]
async fn apply_worker_replays_safely_when_ack_fails_after_apply() {
    let envelope = envelope("tx-ack-fail", "0/16B6E80");
    let first_message = StreamMessage::strict_transaction(&envelope).expect("message");
    let replay_message = StreamMessage::strict_transaction(&envelope).expect("message");
    let first_consumer = RecordingConsumer::failing_first_ack(vec![first_message]);
    let first_consumer_probe = first_consumer.clone();
    let replay_consumer = RecordingConsumer::new(vec![replay_message]);
    let replay_consumer_probe = replay_consumer.clone();
    let applier = RecordingApplier::new(vec![
        Ok(ApplyOutcome {
            decision: ApplyDecision::Applied,
            applied_changes: 1,
            commit_lsn: "0/16B6E80".to_string(),
        }),
        Ok(ApplyOutcome {
            decision: ApplyDecision::SkippedDuplicate,
            applied_changes: 0,
            commit_lsn: "0/16B6E80".to_string(),
        }),
    ]);
    let applier_probe = applier.clone();

    let mut first_worker = ApplyWorker::new(first_consumer, applier.clone());
    let first_result = first_worker.run_once().await;

    assert!(matches!(
        first_result,
        Err(ApplyWorkerError::Stream(StreamError::Consumer(message)))
            if message == "simulated ack failure after apply"
    ));
    assert!(first_consumer_probe.acked_keys().is_empty());

    let mut replay_worker = ApplyWorker::new(replay_consumer, applier);
    let replay_step = replay_worker
        .run_once()
        .await
        .expect("replay step")
        .expect("duplicate replay");

    assert_eq!(replay_step.decision, ApplyDecision::SkippedDuplicate);
    assert_eq!(replay_step.applied_changes, 0);
    assert_eq!(
        replay_consumer_probe.acked_keys(),
        vec!["source:retail:sales:tx-ack-fail:0/16B6E80"]
    );
    assert_eq!(
        applier_probe.applied_transactions(),
        vec!["tx-ack-fail", "tx-ack-fail"]
    );
}
