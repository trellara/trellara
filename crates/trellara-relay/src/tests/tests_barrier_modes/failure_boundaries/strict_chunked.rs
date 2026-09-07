use super::*;

#[tokio::test]
async fn strict_chunked_partial_publish_failure_does_not_advance_checkpoint_or_source_ack() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::failing_on_attempt(4);
    let source = FakeSource::new(vec![multi_change_envelope("tx-large", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::with_mode(
        source,
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::StrictChunked(StrictChunkPlanConfig {
            max_changes_per_chunk: 2,
        }),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Stream(StreamError::Publisher(_)))
    ));

    let messages = publisher.published_messages();
    assert_eq!(messages.len(), 3);
    assert!(messages.iter().all(|message| message.headers.contains(
        &trellara_stream::StreamHeader::new("trellara.message_kind", "strict_chunk")
    )));
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}

#[tokio::test]
async fn strict_chunked_ambiguous_commit_marker_publish_does_not_advance_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::ambiguous_on_attempt(5);
    let source = FakeSource::new(vec![multi_change_envelope("tx-large", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::with_mode(
        source,
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::StrictChunked(StrictChunkPlanConfig {
            max_changes_per_chunk: 2,
        }),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Stream(StreamError::Publisher(_)))
    ));

    let messages = publisher.published_messages();
    assert_eq!(messages.len(), 5);
    assert_eq!(messages[3].topic, "trellara.source.sales.manifest");
    assert_eq!(messages[4].topic, "trellara.source.sales.commit");
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}

#[tokio::test]
async fn strict_chunked_rejects_ddl_before_publish_checkpoint_or_source_ack() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::succeeding();
    let source = FakeSource::new(vec![ddl_envelope("tx-ddl", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::with_mode(
        source,
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::StrictChunked(StrictChunkPlanConfig {
            max_changes_per_chunk: 1,
        }),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Protocol(
            ProtocolError::UnsupportedDdlInManifestMode { boundary_mode, .. }
        )) if boundary_mode == "strict chunked transaction order"
    ));
    assert!(publisher.published_messages().is_empty());
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}
