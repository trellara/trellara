use super::*;

#[tokio::test]
async fn buffered_durability_remains_readable_and_ackable() {
    let root = temp_root("buffered-durability");
    let publisher = LocalPublisher::new(
        LocalPublisherConfig::new(&root).with_durability(LocalDurability::Buffered),
    )
    .expect("publisher");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish");

    let mut consumer = LocalConsumer::new(
        LocalConsumerConfig::new(
            &root,
            "applier",
            vec!["trellara.source.dataset.strict".to_string()],
        )
        .with_durability(LocalDurability::Buffered),
    )
    .expect("consumer");

    let first = consumer.next().await.expect("next").expect("message");
    assert_eq!(first.key, "tx-1");
    consumer.ack(&first).await.expect("ack");
    assert!(consumer.next().await.expect("next").is_none());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn publish_returns_monotonic_offsets_and_persists_messages() {
    let root = temp_root("monotonic");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");

    let first = publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish first");
    let second = publisher
        .publish(message("trellara.source.dataset.strict", "tx-2", "two"))
        .await
        .expect("publish second");

    assert_eq!(first.offset, 0);
    assert_eq!(second.offset, 1);
    assert_eq!(
        indexed_offsets(&root, "trellara.source.dataset.strict").len(),
        2
    );

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");
    let first = consumer.next().await.expect("next").expect("message");
    assert_eq!(first.key, "tx-1");
    assert_eq!(first.payload, Bytes::from_static(b"one"));
    assert_eq!(first.position.as_ref().expect("position").offset, 0);
    consumer.ack(&first).await.expect("ack");
    let second = consumer.next().await.expect("next").expect("message");
    assert_eq!(second.key, "tx-2");
    assert_eq!(second.position.as_ref().expect("position").offset, 1);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn fsync_publish_ack_follows_durable_frame_and_index() {
    let root = temp_root("fsync-publish-ack");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");

    let ack = publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish");

    assert_eq!(ack.topic, topic);
    assert_eq!(ack.partition, 0);
    assert_eq!(ack.offset, 0);
    assert!(topic_path(&root, topic).metadata().expect("log").len() > 0);
    assert_eq!(
        topic_index_path(&root, topic)
            .metadata()
            .expect("index")
            .len(),
        INDEX_ENTRY_BYTES as u64
    );

    let inspection = inspect_local_stream(&root).expect("inspect");
    let inspected_topic = inspection
        .topics
        .iter()
        .find(|candidate| candidate.topic == topic)
        .expect("topic inspection");
    assert_eq!(inspected_topic.message_count, ack.offset + 1);
    assert_eq!(inspected_topic.last_valid_offset, Some(ack.offset));
    assert_eq!(inspected_topic.index_entries, ack.offset + 1);
    assert_eq!(inspected_topic.torn_tail_bytes, 0);
    assert_eq!(inspected_topic.index_status, LocalTopicIndexStatus::Healthy);

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let replayed = consumer.next().await.expect("next").expect("message");
    assert_eq!(replayed.key, "tx-1");
    assert_eq!(replayed.payload, Bytes::from_static(b"one"));
    assert_eq!(
        replayed.position.as_ref().expect("position").offset,
        ack.offset
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_publish_ack_proof_verifies_indexed_replayable_untorn_ack() {
    let root = temp_root("publish-ack-proof");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");

    let ack = publisher
        .publish(message(topic, "tx-proof", "one"))
        .await
        .expect("publish");
    let proof = local_publish_ack_proof(&root, &ack).expect("publish ACK proof");

    assert_eq!(proof.contract, LOCAL_PUBLISH_ACK_PROOF_CONTRACT);
    assert_eq!(proof.durability, "fsync");
    assert!(proof.crash_safe_ack);
    assert_eq!(proof.topic, topic);
    assert_eq!(proof.partition, 0);
    assert_eq!(proof.offset, ack.offset);
    assert_eq!(proof.key, "tx-proof");
    assert!(proof.indexed);
    assert!(proof.replayable);
    assert_eq!(proof.index_status, "healthy");
    assert_eq!(proof.torn_tail_bytes, 0);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn buffered_publish_ack_proof_is_not_crash_safe_ack_evidence() {
    let root = temp_root("buffered-publish-ack-proof");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(
        LocalPublisherConfig::new(&root).with_durability(LocalDurability::Buffered),
    )
    .expect("publisher");

    let ack = publisher
        .publish(message(topic, "tx-buffered-proof", "one"))
        .await
        .expect("publish");
    let proof = local_publish_ack_proof_with_durability(&root, &ack, LocalDurability::Buffered)
        .expect("publish ACK proof");

    assert_eq!(proof.contract, LOCAL_PUBLISH_ACK_PROOF_CONTRACT);
    assert_eq!(proof.durability, "buffered");
    assert!(!proof.crash_safe_ack);
    assert!(proof.indexed);
    assert!(proof.replayable);
    assert_eq!(proof.torn_tail_bytes, 0);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_publish_ack_proof_rejects_non_local_partition_or_missing_offset() {
    let root = temp_root("publish-ack-proof-rejects");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let ack = publisher
        .publish(message(topic, "tx-proof", "one"))
        .await
        .expect("publish");

    let mut wrong_partition = ack.clone();
    wrong_partition.partition = 7;
    assert!(matches!(
        local_publish_ack_proof(&root, &wrong_partition),
        Err(LocalStreamError::InvalidPublishAckProof { topic: rejected, offset: 0, reason })
            if rejected == topic && reason.contains("partition must be 0")
    ));

    let mut missing_offset = ack;
    missing_offset.offset = 4;
    assert!(matches!(
        local_publish_ack_proof(&root, &missing_offset),
        Err(LocalStreamError::InvalidPublishAckProof { topic: rejected, offset: 4, reason })
            if rejected == topic && reason.contains("not present")
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn publish_accepts_zero_partition_and_returns_canonical_local_ack() {
    let root = temp_root("publish-zero-partition");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let mut local_message = message(topic, "tx-1", "one");
    local_message.partition = Some(0);

    let ack = publisher
        .publish(local_message)
        .await
        .expect("publish partition zero");

    assert_eq!(ack.topic, topic);
    assert_eq!(ack.partition, 0);
    assert_eq!(ack.offset, 0);

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let replayed = consumer.next().await.expect("next").expect("message");
    assert_eq!(replayed.partition, Some(0));
    assert_eq!(replayed.position.as_ref().expect("position").partition, 0);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn publish_canonicalizes_nonzero_logical_partition_to_local_ack() {
    let root = temp_root("publish-nonzero-logical-partition");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let mut local_message = message(topic, "tx-1", "one");
    local_message.partition = Some(7);

    let ack = publisher
        .publish(local_message)
        .await
        .expect("publish logical partition");

    assert_eq!(ack.topic, topic);
    assert_eq!(ack.partition, 0);
    assert_eq!(ack.offset, 0);

    let replayed = read_local_message_at(&root, topic, ack.offset)
        .expect("read")
        .expect("message");
    assert_eq!(replayed.partition, Some(0));
    assert_eq!(replayed.position.as_ref().expect("position").partition, 0);

    fs::remove_dir_all(root).expect("cleanup");
}
