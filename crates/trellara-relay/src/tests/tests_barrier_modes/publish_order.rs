use super::*;

#[tokio::test]
async fn partitioned_mode_publishes_chunks_manifest_then_commit_before_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::succeeding();
    let mut relay = Relay::with_mode(
        FakeSource::new(vec![envelope("tx-1", "0/16B6C50")]),
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::Partitioned(PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        }),
    );

    let step = relay
        .run_once()
        .await
        .expect("relay step")
        .expect("published transaction");

    let messages = publisher.published_messages();
    assert_eq!(step.publish_acks.len(), 3);
    assert_eq!(step.source_ack_boundary.expected_publish_messages, 3);
    assert_eq!(step.source_ack_boundary.durable_publish_acks, 3);
    assert!(step.source_ack_boundary.all_publish_acks_durable);
    assert_eq!(messages.len(), 3);
    assert!(messages[0]
        .topic
        .starts_with("trellara.source.sales.partition."));
    assert_eq!(messages[1].topic, "trellara.source.sales.manifest");
    assert!(messages[1]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.message_kind",
            "manifest"
        )));
    assert_eq!(messages[2].topic, "trellara.source.sales.commit");
    assert!(messages[2]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.message_kind",
            "commit_marker"
        )));

    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B6C50");
}

#[tokio::test]
async fn strict_chunked_mode_publishes_chunks_manifest_then_commit_before_checkpoint() {
    let checkpoint_store = InMemoryCheckpointStore::new();
    let publisher = RecordingPublisher::succeeding();
    let mut relay = Relay::with_mode(
        FakeSource::new(vec![multi_change_envelope("tx-large", "0/16B6C50")]),
        publisher.clone(),
        checkpoint_store.clone(),
        RelayMode::StrictChunked(StrictChunkPlanConfig {
            max_changes_per_chunk: 2,
        }),
    );

    let step = relay
        .run_once()
        .await
        .expect("relay step")
        .expect("published transaction");

    let messages = publisher.published_messages();
    assert_eq!(step.publish_acks.len(), 5);
    assert_eq!(step.source_ack_boundary.expected_publish_messages, 5);
    assert_eq!(step.source_ack_boundary.durable_publish_acks, 5);
    assert!(step.source_ack_boundary.all_publish_acks_durable);
    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0].topic, "trellara.source.sales.strict");
    assert_eq!(messages[1].topic, "trellara.source.sales.strict");
    assert_eq!(messages[2].topic, "trellara.source.sales.strict");
    assert!(messages[0]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.message_kind",
            "strict_chunk"
        )));
    assert_eq!(messages[3].topic, "trellara.source.sales.manifest");
    assert!(messages[3]
        .headers
        .contains(&trellara_stream::StreamHeader::new(
            "trellara.global_event_count",
            "5"
        )));
    assert_eq!(messages[4].topic, "trellara.source.sales.commit");

    let checkpoint = load_source_checkpoint(&checkpoint_store, "source", "sales")
        .await
        .expect("checkpoint load")
        .expect("checkpoint");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B6C50");
}
