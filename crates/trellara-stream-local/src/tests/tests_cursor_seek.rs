use super::*;

#[tokio::test]
async fn set_local_cursor_rewinds_consumer_for_replay() {
    let root = temp_root("seek-replay");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish first");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-2", "two"))
        .await
        .expect("publish second");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");
    let first = consumer.next().await.expect("next").expect("message");
    consumer.ack(&first).await.expect("ack");
    assert_eq!(first.key, "tx-1");

    let cursor = set_local_cursor(&root, "applier", "trellara.source.dataset.strict", 0)
        .expect("seek cursor");
    assert_eq!(cursor.next_offset, 0);

    let replayed = consumer.next().await.expect("next").expect("message");
    assert_eq!(replayed.key, "tx-1");
    assert_eq!(replayed.position.as_ref().expect("position").offset, 0);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn read_local_message_at_returns_position_without_cursor_side_effects() {
    let root = temp_root("read-message-at");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");
    publisher
        .publish(message(topic, "tx-2", "two"))
        .await
        .expect("publish second");

    let found = read_local_message_at(&root, topic, 1)
        .expect("read offset")
        .expect("message at offset");

    assert_eq!(found.key, "tx-2");
    assert_eq!(found.payload, Bytes::from_static(b"two"));
    assert_eq!(
        found.position,
        Some(StreamPosition {
            topic: topic.to_string(),
            partition: 0,
            offset: 1,
        })
    );
    assert!(!cursor_path(&root, "applier", topic).exists());
    assert!(read_local_message_at(&root, topic, 2)
        .expect("read beyond topic")
        .is_none());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn set_local_cursor_rejects_accidental_fast_forward_past_available_messages() {
    let root = temp_root("seek-forward");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish");

    let error = set_local_cursor(&root, "applier", "trellara.source.dataset.strict", 5)
        .expect_err("reject accidental fast-forward");
    assert!(matches!(
        error,
        LocalStreamError::CursorOffsetAhead {
            offset: 5,
            message_count: 1,
            ..
        }
    ));
    assert!(!cursor_path(&root, "applier", "trellara.source.dataset.strict").exists());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn set_local_cursor_allows_explicit_fast_forward_for_repair() {
    let root = temp_root("seek-forward-allowed");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish");

    let cursor =
        set_local_cursor_with_policy(&root, "applier", "trellara.source.dataset.strict", 5, true)
            .expect("seek cursor with explicit fast-forward");
    assert_eq!(cursor.status, LocalCursorStatus::AheadOfTopic);
    assert_eq!(cursor.topic_message_count, Some(1));
    assert_eq!(cursor.pending_messages, Some(0));
    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");

    assert!(consumer.next().await.expect("next").is_none());

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn set_local_cursor_rejects_negative_offsets() {
    assert!(matches!(
        set_local_cursor(
            temp_root("seek-negative"),
            "applier",
            "trellara.source.dataset.strict",
            -1,
        ),
        Err(LocalStreamError::NegativeCursorOffset { offset: -1 })
    ));
}
