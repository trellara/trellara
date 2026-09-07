use super::*;

#[tokio::test]
async fn partitioned_partial_publish_failure_does_not_advance_checkpoint_or_source_ack() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::failing_on_attempt(2);
    let source = FakeSource::new(vec![envelope("tx-partitioned", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::with_mode(
        source,
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::Partitioned(PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        }),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Stream(StreamError::Publisher(_)))
    ));

    let messages = publisher.published_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0]
        .topic
        .starts_with("trellara.source.sales.partition."));
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}

#[tokio::test]
async fn partitioned_ambiguous_commit_marker_publish_does_not_advance_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::ambiguous_on_attempt(3);
    let source = FakeSource::new(vec![envelope("tx-partitioned", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::with_mode(
        source,
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::Partitioned(PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        }),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Stream(StreamError::Publisher(_)))
    ));

    let messages = publisher.published_messages();
    assert_eq!(messages.len(), 3);
    assert!(messages[0]
        .topic
        .starts_with("trellara.source.sales.partition."));
    assert_eq!(messages[1].topic, "trellara.source.sales.manifest");
    assert_eq!(messages[2].topic, "trellara.source.sales.commit");
    assert!(messages[2]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.message_kind",
            "commit_marker"
        )));
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}

#[tokio::test]
async fn partitioned_rejects_ddl_before_publish_checkpoint_or_source_ack() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::succeeding();
    let source = FakeSource::new(vec![ddl_envelope("tx-ddl", "0/16B6C50")]);
    let acked_lsns = source.acked_lsns();
    let mut relay = Relay::with_mode(
        source,
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::Partitioned(PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        }),
    );

    assert!(matches!(
        relay.run_once().await,
        Err(RelayError::Protocol(
            ProtocolError::UnsupportedDdlInManifestMode { boundary_mode, .. }
        )) if boundary_mode == "partitioned scale mode"
    ));
    assert!(publisher.published_messages().is_empty());
    assert!(load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .is_none());
    assert!(acked_lsns.lock().expect("acked lsn lock").is_empty());
}
