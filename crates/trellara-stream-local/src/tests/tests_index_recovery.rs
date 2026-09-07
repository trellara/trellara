use super::*;

#[tokio::test]
async fn missing_index_is_rebuilt_for_offset_replay() {
    let root = temp_root("missing-index");
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
    fs::remove_file(topic_index_path(&root, topic)).expect("remove index");

    set_local_cursor(&root, "applier", topic, 1).expect("seek cursor");
    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");

    let second = consumer.next().await.expect("next").expect("message");
    assert_eq!(second.key, "tx-2");
    assert_eq!(second.position.as_ref().expect("position").offset, 1);
    assert_eq!(indexed_offsets(&root, topic).len(), 2);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn inspect_reports_missing_index_rebuild() {
    let root = temp_root("inspect-missing-index");
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
    fs::remove_file(topic_index_path(&root, topic)).expect("remove index");

    let inspection = inspect_local_stream(&root).expect("inspect");

    assert_eq!(inspection.topics[0].index_entries, 2);
    assert_eq!(inspection.topics[0].index_bytes, 16);
    assert_eq!(
        inspection.topics[0].index_status,
        LocalTopicIndexStatus::MissingRebuilt
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn stale_index_discovers_durable_tail_without_truncating() {
    let root = temp_root("stale-index");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");
    let first_only_index = indexed_offsets(&root, topic);
    publisher
        .publish(message(topic, "tx-2", "two"))
        .await
        .expect("publish second");
    write_index(
        &topic_index_path(&root, topic),
        &first_only_index,
        LocalDurability::Buffered,
    )
    .expect("write stale index");

    set_local_cursor(&root, "applier", topic, 1).expect("seek cursor");
    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");

    let second = consumer.next().await.expect("next").expect("message");
    assert_eq!(second.key, "tx-2");
    assert_eq!(second.payload, Bytes::from_static(b"two"));
    assert_eq!(indexed_offsets(&root, topic).len(), 2);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn inspect_reports_corrupt_index_rebuild() {
    let root = temp_root("inspect-corrupt-index");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");
    fs::write(topic_index_path(&root, topic), [1, 2, 3]).expect("write corrupt index");

    let inspection = inspect_local_stream(&root).expect("inspect");

    assert_eq!(inspection.topics[0].index_entries, 1);
    assert_eq!(inspection.topics[0].index_bytes, 8);
    assert_eq!(
        inspection.topics[0].index_status,
        LocalTopicIndexStatus::CorruptRebuilt
    );

    fs::remove_dir_all(root).expect("cleanup");
}
