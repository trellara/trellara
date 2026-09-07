use super::*;
use std::io::Write;

use crate::frame::FRAME_MAGIC;

#[tokio::test]
async fn recovery_inspection_reports_ready_stream_with_recovered_index_evidence() {
    let root = temp_root("recovery-ready-rebuilt-index");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");
    fs::remove_file(topic_index_path(&root, topic)).expect("remove index");

    let recovery = inspect_local_recovery(&root).expect("recovery inspection");

    assert!(recovery.recovery_ready);
    assert_eq!(recovery.topic_count, 1);
    assert_eq!(recovery.total_message_count, 1);
    assert_eq!(recovery.replayable_topic_count, 1);
    assert!(recovery.torn_tail_topics.is_empty());
    assert_eq!(recovery.recovered_index_topics.len(), 1);
    assert_eq!(recovery.recovered_index_topics[0].topic, topic);
    assert_eq!(
        recovery.recovered_index_topics[0].index_status,
        LocalTopicIndexStatus::MissingRebuilt.as_str()
    );
    assert_eq!(recovery.recovery_actions.len(), 1);
    assert_eq!(
        recovery.recovery_actions[0].code,
        "index_rebuilt_from_segment_log"
    );
    assert_eq!(
        recovery.recovery_actions[0].command,
        format!("trellara stream inspect-local --config <flow> --topic {topic}")
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn recovery_inspection_blocks_torn_tail_until_next_recovery_append() {
    let root = temp_root("recovery-torn-tail-blocker");
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

    let recovery = inspect_local_recovery(&root).expect("recovery inspection");

    assert!(!recovery.recovery_ready);
    assert_eq!(recovery.torn_tail_topics.len(), 1);
    let torn = &recovery.torn_tail_topics[0];
    assert_eq!(torn.topic, topic);
    assert_eq!(torn.message_count, 1);
    assert_eq!(torn.last_valid_offset, Some(0));
    assert_eq!(torn.valid_bytes, valid_bytes);
    assert_eq!(torn.file_bytes, valid_bytes + 8);
    assert_eq!(torn.torn_tail_bytes, 8);
    assert_eq!(torn.index_status, LocalTopicIndexStatus::Healthy.as_str());
    assert_eq!(recovery.recovery_actions.len(), 1);
    let action = &recovery.recovery_actions[0];
    assert_eq!(action.code, "truncate_torn_tail_on_next_publish");
    assert_eq!(action.topic.as_deref(), Some(topic));
    assert_eq!(action.command, "trellara relay --config <flow>");
    assert!(action.reason.contains("8 torn tail bytes"));

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn recovery_inspection_blocks_cursors_that_cannot_replay() {
    let root = temp_root("recovery-cursor-blocker");
    let topic = "trellara.source.dataset.strict";
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(topic, "tx-1", "one"))
        .await
        .expect("publish first");
    set_local_cursor_with_policy(&root, "applier", topic, 2, true).expect("force cursor ahead");

    let recovery = inspect_local_recovery(&root).expect("recovery inspection");

    assert!(!recovery.recovery_ready);
    assert_eq!(
        recovery.cursor_blockers,
        vec![LocalRecoveryCursorBlocker {
            group_id: "applier".to_string(),
            topic: topic.to_string(),
            next_offset: 2,
            status: LocalCursorStatus::AheadOfTopic.as_str(),
        }]
    );
    assert_eq!(recovery.recovery_actions.len(), 1);
    assert_eq!(recovery.recovery_actions[0].code, "repair_cursor_blocker");
    assert_eq!(
        recovery.recovery_actions[0].command,
        "trellara stream seek-local --config <flow> --consumer-group applier --topic trellara.source.dataset.strict --next-offset <last-valid-offset>"
    );

    fs::remove_dir_all(root).expect("cleanup");
}
