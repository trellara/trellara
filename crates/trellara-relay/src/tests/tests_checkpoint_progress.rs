use super::*;

#[tokio::test]
async fn stale_replay_does_not_move_source_checkpoint_or_ack_backwards() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    checkpoint_store
        .save_checkpoint(Checkpoint {
            source_id: "source".to_string(),
            dataset_id: "sales".to_string(),
            last_seen_lsn: "0/16B8000".to_string(),
            last_durable_lsn: "0/16B8000".to_string(),
            last_applied_lsn: "0/16B7000".to_string(),
        })
        .await
        .expect("seed checkpoint");
    let publisher = RecordingPublisher::succeeding();
    let source = FakeSource::new(vec![envelope("tx-stale", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::new(source, publisher.clone(), checkpoint_store.clone());

    let step = relay
        .run_once()
        .await
        .expect("relay step")
        .expect("published stale transaction");

    assert_eq!(publisher.published_messages().len(), 1);
    assert_eq!(step.source_ack_lsn, "0/16B8000");
    assert_eq!(
        *acked_lsns.lock().expect("acked lsn lock"),
        vec!["0/16B8000".to_string()]
    );
    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B8000");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B8000");
    assert_eq!(checkpoint.last_applied_lsn, "0/16B7000");
}

#[tokio::test]
async fn newer_relay_checkpoint_preserves_existing_target_applied_lsn() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    checkpoint_store
        .save_checkpoint(Checkpoint {
            source_id: "source".to_string(),
            dataset_id: "sales".to_string(),
            last_seen_lsn: "0/16B7000".to_string(),
            last_durable_lsn: "0/16B7000".to_string(),
            last_applied_lsn: "0/16B6800".to_string(),
        })
        .await
        .expect("seed checkpoint");
    let mut relay = Relay::new(
        FakeSource::new(vec![envelope("tx-newer", "0/16B8000")]),
        RecordingPublisher::succeeding(),
        checkpoint_store.clone(),
    );

    let step = relay
        .run_once()
        .await
        .expect("relay step")
        .expect("published newer transaction");

    assert_eq!(step.source_ack_lsn, "0/16B8000");
    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B8000");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B8000");
    assert_eq!(checkpoint.last_applied_lsn, "0/16B6800");
}
