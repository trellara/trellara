use super::*;
use trellara_checkpoint::InMemoryCheckpointStore;

#[tokio::test]
async fn ambiguous_publish_replays_duplicate_without_advancing_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let first_publisher = RecordingPublisher::ambiguous_after_broker_accept();
    let envelope = envelope("tx-ambiguous", "0/16B6C50");
    let mut first_relay = Relay::new(
        FakeSource::new(vec![envelope.clone()]),
        first_publisher.clone(),
        checkpoint_store.clone(),
    );

    assert!(matches!(
        first_relay.run_once().await,
        Err(RelayError::Stream(StreamError::Publisher(_)))
    ));
    let first_messages = first_publisher.published_messages();
    assert_eq!(first_messages.len(), 1);
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());

    let recovery_publisher = RecordingPublisher::succeeding();
    let mut recovery_relay = Relay::new(
        FakeSource::new(vec![envelope]),
        recovery_publisher.clone(),
        checkpoint_store.clone(),
    );
    recovery_relay
        .run_once()
        .await
        .expect("recovery publish")
        .expect("replayed transaction");

    let recovery_messages = recovery_publisher.published_messages();
    assert_eq!(recovery_messages.len(), 1);
    assert_eq!(first_messages[0].key, recovery_messages[0].key);
    assert_eq!(first_messages[0].headers, recovery_messages[0].headers);
    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B6C50");
}
