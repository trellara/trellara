use super::*;

#[tokio::test]
async fn unacked_message_is_replayed_after_consumer_restart() {
    let root = temp_root("unacked");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish");

    let mut first_consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");
    let first = first_consumer.next().await.expect("next").expect("message");
    assert_eq!(first.key, "tx-1");

    let mut restarted = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");
    let replayed = restarted.next().await.expect("next").expect("message");
    assert_eq!(replayed.key, "tx-1");
    assert_eq!(replayed.position.as_ref().expect("position").offset, 0);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn acked_message_is_not_replayed_after_consumer_restart() {
    let root = temp_root("acked");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let first = consumer.next().await.expect("next").expect("message");
    consumer.ack(&first).await.expect("ack");

    let mut restarted = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    assert!(restarted.next().await.expect("next").is_none());
    assert_eq!(read_cursor(&restarted.config, topic).expect("cursor"), 1);
    assert!(cursor_path(&root, "applier", topic).exists());
    assert!(!cursor_path(&root, "applier", topic)
        .with_extension("tmp")
        .exists());
    assert!(topic_index_path(&root, topic).exists());
    assert!(!topic_index_path(&root, topic)
        .with_extension("idx.tmp")
        .exists());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn live_consumer_can_read_ahead_without_advancing_durable_cursor() {
    let root = temp_root("read-ahead");
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

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let first = consumer.next().await.expect("first next").expect("first");
    let second = consumer.next().await.expect("second next").expect("second");

    assert_eq!(first.key, "tx-1");
    assert_eq!(first.position.as_ref().expect("first position").offset, 0);
    assert_eq!(second.key, "tx-2");
    assert_eq!(second.position.as_ref().expect("second position").offset, 1);
    assert_eq!(read_cursor(&consumer.config, topic).expect("cursor"), 0);

    let mut restarted = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let replayed = restarted
        .next()
        .await
        .expect("replay next")
        .expect("replay");
    assert_eq!(replayed.key, "tx-1");
    assert_eq!(replayed.position.as_ref().expect("position").offset, 0);

    fs::remove_dir_all(root).expect("cleanup");
}
