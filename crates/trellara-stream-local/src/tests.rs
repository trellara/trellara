use super::*;
use std::path::PathBuf;

use bytes::Bytes;
use trellara_stream::{StreamHeader, StreamMessage};

use crate::index::{read_index, IndexRead, INDEX_ENTRY_BYTES};
use crate::paths::cursor_path;

mod tests_ack_cursor;
mod tests_barrier_reconstruct;
mod tests_consumer_replay;
mod tests_cursor_seek;
mod tests_cursor_validation;
mod tests_frame_format;
mod tests_frame_headers;
mod tests_index_recovery;
mod tests_inspection_cursor;
mod tests_publish_consume;
mod tests_publish_durability;
mod tests_publish_validation;
mod tests_recovery_inspection;
mod tests_source_ack_proof;
mod tests_torn_tail;

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "trellara-stream-local-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}

fn message(topic: &str, key: &str, payload: &str) -> StreamMessage {
    StreamMessage {
        topic: topic.to_string(),
        key: key.to_string(),
        payload: Bytes::copy_from_slice(payload.as_bytes()),
        headers: vec![StreamHeader::new("trellara.transaction_id", key)],
        partition: None,
        position: None,
    }
}

fn indexed_offsets(root: &Path, topic: &str) -> Vec<u64> {
    let log = topic_path(root, topic);
    let log_len = log.metadata().expect("log metadata").len();
    match read_index(&topic_index_path(root, topic), log_len).expect("read index") {
        IndexRead::Valid(positions) => positions,
        IndexRead::Missing => panic!("index is missing"),
        IndexRead::Corrupt => panic!("index is corrupt"),
    }
}

#[test]
fn embedded_transport_design_doc_matches_local_stream_contract() {
    let design = include_str!("../../../docs/DESIGN.md");

    assert!(design.contains("Embedded Transport Design"));
    assert!(design.contains("StreamPublisher"));
    assert!(design.contains("StreamConsumer"));
    assert!(design.contains("TLG2"));
    assert!(design.contains("LocalDurability::Fsync"));
    assert!(design.contains("missing indexes are rebuilt"));
    assert!(design.contains("torn tail"));
    assert!(design.contains("cursor is persisted atomically"));
    assert!(design.contains("source acknowledgement after local durability"));
    assert!(design.contains("stream locate-local"));
    assert!(design.contains("stream seek-local"));
}

#[test]
fn config_defaults_to_fsync_durability() {
    let root = temp_root("default-durability");
    let publisher = LocalPublisherConfig::new(&root);
    let consumer = LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    );

    assert_eq!(publisher.durability, LocalDurability::Fsync);
    assert_eq!(consumer.durability, LocalDurability::Fsync);
}
