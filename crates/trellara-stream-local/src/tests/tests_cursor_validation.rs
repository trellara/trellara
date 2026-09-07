use super::*;

#[tokio::test]
async fn consumer_rejects_corrupt_cursor_file() {
    let root = temp_root("corrupt-cursor");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish");
    let cursor_file = cursor_path(&root, "applier", topic);
    fs::create_dir_all(cursor_file.parent().expect("cursor parent")).expect("cursor dir");
    fs::write(&cursor_file, "not-an-offset").expect("write corrupt cursor");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let error = next_local(&mut consumer).expect_err("corrupt cursor");
    assert!(matches!(error, LocalStreamError::InvalidCursor { .. }));
    let error = inspect_local_stream(&root).expect_err("inspect corrupt cursor");
    assert!(matches!(error, LocalStreamError::InvalidCursor { .. }));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn consumer_rejects_padded_cursor_file() {
    let root = temp_root("padded-cursor");
    let topic = "trellara.source.dataset.strict";
    let cursor_file = cursor_path(&root, "applier", topic);
    fs::create_dir_all(cursor_file.parent().expect("cursor parent")).expect("cursor dir");
    fs::write(&cursor_file, " 1\n").expect("write padded cursor");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let error = next_local(&mut consumer).expect_err("padded cursor");

    assert!(matches!(
        error,
        LocalStreamError::InvalidCursor { value, .. } if value == " 1\n"
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn consumer_rejects_negative_cursor_file() {
    let root = temp_root("negative-cursor-file");
    let topic = "trellara.source.dataset.strict";
    let cursor_file = cursor_path(&root, "applier", topic);
    fs::create_dir_all(cursor_file.parent().expect("cursor parent")).expect("cursor dir");
    fs::write(&cursor_file, "-2").expect("write negative cursor");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let error = next_local(&mut consumer).expect_err("negative cursor");

    assert!(matches!(
        error,
        LocalStreamError::NegativeCursorOffset { offset: -2 }
    ));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn ack_rejects_negative_message_position() {
    let root = temp_root("negative-ack");
    let consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");
    let mut message = message("trellara.source.dataset.strict", "tx-1", "one");
    message.position = Some(StreamPosition {
        topic: "trellara.source.dataset.strict".to_string(),
        partition: 0,
        offset: -1,
    });

    let error = ack_local(&consumer.config, &message).expect_err("negative ack");

    assert!(matches!(
        error,
        LocalStreamError::NegativeCursorOffset { offset: -1 }
    ));
    assert!(!cursor_path(&root, "applier", "trellara.source.dataset.strict").exists());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn ack_rejects_nonzero_local_partition() {
    let root = temp_root("ack-partition");
    let topic = "trellara.source.dataset.strict";
    let consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let mut message = message(topic, "tx-1", "one");
    message.position = Some(StreamPosition {
        topic: topic.to_string(),
        partition: 1,
        offset: 0,
    });

    let error = ack_local(&consumer.config, &message).expect_err("partition ack");

    assert!(matches!(
        error,
        LocalStreamError::UnsupportedAckPartition {
            topic: ref error_topic,
            partition: 1
        } if error_topic == topic
    ));
    assert!(!cursor_path(&root, "applier", topic).exists());

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn consumer_rejects_empty_topic_list() {
    let root = temp_root("missing-topics");
    let error = LocalConsumer::new(LocalConsumerConfig::new(&root, "applier", Vec::new()))
        .expect_err("missing topics");

    assert!(matches!(error, LocalStreamError::MissingTopics));
}
