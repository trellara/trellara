use super::*;

#[tokio::test]
async fn ack_rejects_non_contiguous_cursor_advance() {
    let root = temp_root("ack-non-contiguous");
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

    let consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let mut second = message(topic, "tx-2", "two");
    second.position = Some(StreamPosition {
        topic: topic.to_string(),
        partition: 0,
        offset: 1,
    });

    let error = ack_local(&consumer.config, &second).expect_err("non-contiguous ack");

    assert!(matches!(
        error,
        LocalStreamError::NonContiguousAck {
            ref topic,
            offset: 1,
            current_next_offset: 0,
        } if topic == "trellara.source.dataset.strict"
    ));

    let inspection = inspect_local_stream(&root).expect("inspect");
    assert!(inspection.cursors.is_empty());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn ack_outcome_marks_exact_duplicate_ack_as_noop() {
    let root = temp_root("ack-duplicate-noop");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");

    let consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let mut first = message(topic, "tx-1", "one");
    first.position = Some(StreamPosition {
        topic: topic.to_string(),
        partition: 0,
        offset: 0,
    });

    let first_ack = ack_local(&consumer.config, &first).expect("ack first");
    let duplicate_ack = ack_local(&consumer.config, &first).expect("ack duplicate");

    assert!(first_ack.advanced);
    assert_eq!(duplicate_ack.previous_next_offset, 1);
    assert_eq!(duplicate_ack.acked_next_offset, 1);
    assert_eq!(duplicate_ack.durable_next_offset, 1);
    assert!(!duplicate_ack.advanced);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn consumer_ack_with_outcome_reports_durable_cursor_progress() {
    let root = temp_root("ack-with-outcome");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let first = consumer
        .next()
        .await
        .expect("read first")
        .expect("first message");

    let first_ack = consumer
        .ack_with_outcome(&first)
        .expect("ack first with outcome");
    let duplicate_ack = consumer
        .ack_with_outcome(&first)
        .expect("ack duplicate with outcome");

    assert_eq!(first_ack.previous_next_offset, 0);
    assert_eq!(first_ack.acked_next_offset, 1);
    assert_eq!(first_ack.durable_next_offset, 1);
    assert!(first_ack.advanced);
    assert_eq!(duplicate_ack.previous_next_offset, 1);
    assert_eq!(duplicate_ack.acked_next_offset, 1);
    assert_eq!(duplicate_ack.durable_next_offset, 1);
    assert!(!duplicate_ack.advanced);

    let inspection = inspect_local_stream(&root).expect("inspect");
    assert_eq!(
        inspection.cursors,
        vec![LocalCursorInspection::from_topic_depth(
            "applier",
            topic,
            1,
            Some(1),
        )]
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn ack_rejects_cursor_offset_overflow() {
    let root = temp_root("ack-overflow");
    let topic = "trellara.source.dataset.strict";
    let consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let mut message = message(topic, "tx-max", "max");
    message.position = Some(StreamPosition {
        topic: topic.to_string(),
        partition: 0,
        offset: i64::MAX,
    });

    let error = ack_local(&consumer.config, &message).expect_err("ack overflow");

    assert!(matches!(
        error,
        LocalStreamError::CursorOffsetOverflow { offset: i64::MAX }
    ));
    assert!(!cursor_path(&root, "applier", topic).exists());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn ack_rejects_future_offset_beyond_durable_topic_depth() {
    let root = temp_root("ack-future-offset");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish");

    let consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let mut message = message(topic, "tx-future", "future");
    message.position = Some(StreamPosition {
        topic: topic.to_string(),
        partition: 0,
        offset: 1,
    });

    let error = ack_local(&consumer.config, &message).expect_err("future ack");

    assert!(matches!(
        error,
        LocalStreamError::CursorOffsetAhead {
            ref topic,
            offset: 1,
            message_count: 1,
        } if topic == "trellara.source.dataset.strict"
    ));
    assert!(!cursor_path(&root, "applier", topic).exists());

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn ack_rejects_missing_topic_position_without_creating_cursor() {
    let root = temp_root("ack-missing-topic");
    let topic = "trellara.source.dataset.strict";
    let consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let mut message = message(topic, "tx-missing", "missing");
    message.position = Some(StreamPosition {
        topic: topic.to_string(),
        partition: 0,
        offset: 0,
    });

    let error = ack_local(&consumer.config, &message).expect_err("missing topic ack");

    assert!(matches!(
        error,
        LocalStreamError::CursorOffsetAhead {
            ref topic,
            offset: 0,
            message_count: 0,
        } if topic == "trellara.source.dataset.strict"
    ));
    assert!(!cursor_path(&root, "applier", topic).exists());

    fs::remove_dir_all(root).expect("cleanup");
}
