use super::*;
use std::io::Write;

use crate::frame::FRAME_MAGIC;

#[tokio::test]
async fn publisher_truncates_torn_tail_before_next_append() {
    let root = temp_root("torn-tail");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish first");
    let path = topic_path(&root, "trellara.source.dataset.strict");
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open topic");
        file.write_all(FRAME_MAGIC).expect("write partial magic");
        file.write_all(&99_u32.to_le_bytes())
            .expect("write partial len");
        file.sync_all().expect("sync torn tail");
    }

    let second = publisher
        .publish(message("trellara.source.dataset.strict", "tx-2", "two"))
        .await
        .expect("publish second");

    assert_eq!(second.offset, 1);
    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");
    let first = consumer.next().await.expect("next").expect("first");
    assert_eq!(first.key, "tx-1");
    consumer.ack(&first).await.expect("ack first");
    let second = consumer.next().await.expect("next").expect("second");
    assert_eq!(second.key, "tx-2");
    consumer.ack(&second).await.expect("ack second");
    assert!(consumer.next().await.expect("next").is_none());
    assert_eq!(
        indexed_offsets(&root, "trellara.source.dataset.strict").len(),
        2
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn inspect_reports_torn_tail_bytes_before_recovery_append() {
    let root = temp_root("inspect-torn-tail");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");
    let path = topic_path(&root, topic);
    let valid_bytes = fs::metadata(&path).expect("topic metadata").len();
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open topic");
        file.write_all(FRAME_MAGIC).expect("write partial magic");
        file.write_all(&99_u32.to_le_bytes())
            .expect("write partial len");
        file.sync_all().expect("sync torn tail");
    }

    let inspection = inspect_local_stream(&root).expect("inspect");
    let topic = &inspection.topics[0];

    assert_eq!(topic.message_count, 1);
    assert_eq!(topic.last_valid_offset, Some(0));
    assert!(topic.replayable);
    assert_eq!(topic.valid_bytes, valid_bytes);
    assert_eq!(topic.file_bytes, valid_bytes + 8);
    assert_eq!(topic.torn_tail_bytes, 8);
    assert_eq!(topic.index_status, LocalTopicIndexStatus::Healthy);

    fs::remove_dir_all(root).expect("cleanup");
}
