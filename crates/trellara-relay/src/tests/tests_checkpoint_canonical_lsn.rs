use super::*;

#[tokio::test]
async fn relay_canonicalizes_checkpoint_and_source_ack_lsn() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let source = FakeSource::new(vec![envelope("tx-canonical", "00000000/016B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::new(
        source,
        RecordingPublisher::succeeding(),
        checkpoint_store.clone(),
    );

    let step = relay
        .run_once()
        .await
        .expect("relay step")
        .expect("published canonical transaction");

    assert_eq!(step.source_ack_lsn, "0/16B6C50");
    assert_eq!(step.source_ack_boundary.commit_lsn, "0/16B6C50");
    assert_eq!(step.source_ack_boundary.source_ack_lsn, "0/16B6C50");
    assert_eq!(
        *acked_lsns.lock().expect("acked lsn lock"),
        vec!["0/16B6C50".to_string()]
    );
    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B6C50");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B6C50");
}
