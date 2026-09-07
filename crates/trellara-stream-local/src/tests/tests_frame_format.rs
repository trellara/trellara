use super::*;
use std::fs::File;
use std::io::Write;

use bytes::Bytes;
use trellara_stream::StreamMessage;

use crate::frame::{FRAME_MAGIC, LEGACY_FRAME_MAGIC};
use crate::frame_limits::MAX_FRAME_BODY_BYTES;

fn write_legacy_frame(file: &mut File, message: &StreamMessage) {
    file.write_all(LEGACY_FRAME_MAGIC)
        .expect("write legacy magic");
    write_legacy_bytes(file, message.key.as_bytes());
    write_legacy_bytes(file, &message.payload);
    file.write_all(&(message.headers.len() as u32).to_le_bytes())
        .expect("write legacy header count");
    for header in &message.headers {
        write_legacy_bytes(file, header.key.as_bytes());
        write_legacy_bytes(file, header.value.as_bytes());
    }
}

fn write_legacy_bytes(file: &mut File, bytes: &[u8]) {
    file.write_all(&(bytes.len() as u32).to_le_bytes())
        .expect("write legacy len");
    file.write_all(bytes).expect("write legacy bytes");
}

#[tokio::test]
async fn publisher_writes_crc_checked_frames() {
    let root = temp_root("crc-frame");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish");
    let path = topic_path(&root, topic);
    let bytes = fs::read(&path).expect("read topic");

    assert_eq!(&bytes[..4], FRAME_MAGIC);
    assert!(bytes.len() > 12);

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn consumer_rejects_complete_frame_with_crc_mismatch() {
    let root = temp_root("crc-mismatch");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish");
    let path = topic_path(&root, topic);
    let mut bytes = fs::read(&path).expect("read topic");
    let payload_position = bytes
        .windows(3)
        .position(|window| window == b"one")
        .expect("payload position");
    bytes[payload_position] ^= 0xff;
    fs::write(&path, bytes).expect("tamper topic");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let error = next_local(&mut consumer).expect_err("crc mismatch");

    assert!(matches!(error, LocalStreamError::CorruptFrame { .. }));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn consumer_rejects_absurd_frame_length_before_allocation() {
    let root = temp_root("absurd-frame-length");
    let topic = "trellara.source.dataset.strict";
    fs::create_dir_all(topic_dir(&root)).expect("topic dir");
    let path = topic_path(&root, topic);
    let mut file = File::create(&path).expect("topic");
    file.write_all(FRAME_MAGIC).expect("write magic");
    file.write_all(&(MAX_FRAME_BODY_BYTES + 1).to_le_bytes())
        .expect("write absurd length");
    file.sync_all().expect("sync topic");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let error = next_local(&mut consumer).expect_err("absurd frame length");

    assert!(matches!(error, LocalStreamError::CorruptFrame { .. }));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn consumer_rejects_absurd_legacy_field_length_before_allocation() {
    let root = temp_root("absurd-legacy-field-length");
    let topic = "trellara.source.dataset.strict";
    fs::create_dir_all(topic_dir(&root)).expect("topic dir");
    let path = topic_path(&root, topic);
    let mut file = File::create(&path).expect("topic");
    file.write_all(LEGACY_FRAME_MAGIC)
        .expect("write legacy magic");
    file.write_all(&(MAX_FRAME_BODY_BYTES + 1).to_le_bytes())
        .expect("write absurd key length");
    file.sync_all().expect("sync topic");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let error = next_local(&mut consumer).expect_err("absurd legacy field length");

    assert!(matches!(error, LocalStreamError::CorruptFrame { .. }));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn consumer_replays_legacy_frames_without_crc() {
    let root = temp_root("legacy-frame");
    let topic = "trellara.source.dataset.strict";
    fs::create_dir_all(topic_dir(&root)).expect("topic dir");
    let mut file = File::create(topic_path(&root, topic)).expect("legacy topic");
    write_legacy_frame(&mut file, &message(topic, "tx-legacy", "legacy"));
    file.sync_all().expect("sync legacy topic");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![topic.to_string()],
    ))
    .expect("consumer");
    let replayed = consumer.next().await.expect("next").expect("message");

    assert_eq!(replayed.key, "tx-legacy");
    assert_eq!(replayed.payload, Bytes::from_static(b"legacy"));
    assert_eq!(replayed.position.as_ref().expect("position").offset, 0);

    fs::remove_dir_all(root).expect("cleanup");
}
