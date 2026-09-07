use super::*;
use trellara_stream_local::LocalTopicProofHeader;

#[test]
fn local_stream_inspect_summary_reports_depth_and_pending_messages() {
    let summary = LocalStreamInspectSummary::from_inspection(
        LocalStreamInspection {
            root: PathBuf::from("/tmp/trellara-local-stream"),
            topics: vec![LocalTopicInspection {
                topic: "trellara.local-source.retail-sales.strict".to_string(),
                message_count: 3,
                last_valid_offset: Some(2),
                last_proof_headers: vec![LocalTopicProofHeader {
                    key: "trellara.ddl_target_ack_required".to_string(),
                    value: "1".to_string(),
                }],
                replayable: true,
                valid_bytes: 90,
                file_bytes: 94,
                torn_tail_bytes: 4,
                index_entries: 3,
                index_bytes: 24,
                index_status: trellara_stream_local::LocalTopicIndexStatus::StaleRebuilt,
            }],
            cursors: vec![
                LocalCursorInspection::from_topic_depth(
                    "applier",
                    "trellara.local-source.retail-sales.strict",
                    2,
                    Some(3),
                ),
                LocalCursorInspection::from_topic_depth(
                    "ahead",
                    "trellara.local-source.retail-sales.strict",
                    9,
                    Some(3),
                ),
                LocalCursorInspection::from_topic_depth("orphan", "missing.topic", 7, None),
            ],
        },
        vec!["trellara.local-source.retail-sales.strict".to_string()],
    )
    .expect("summary");

    assert_eq!(summary.root, "/tmp/trellara-local-stream");
    assert_eq!(summary.configured_topics.len(), 1);
    assert!(!summary.barrier_topics.required);
    assert_eq!(summary.barrier_topics.expected_topic_count, 1);
    assert_eq!(summary.barrier_topics.observed_configured_topic_count, 1);
    assert!(summary.barrier_topics.missing_topics.is_empty());
    assert_eq!(summary.barrier_topics.manifest_topic_present, None);
    assert_eq!(summary.barrier_topics.commit_topic_present, None);
    assert_eq!(summary.health.status, "degraded");
    assert_eq!(summary.health.topic_count, 1);
    assert_eq!(summary.health.cursor_count, 3);
    assert_eq!(summary.health.total_messages, 3);
    assert_eq!(summary.health.total_pending_messages, 1);
    assert_eq!(summary.health.total_valid_bytes, 90);
    assert_eq!(summary.health.total_file_bytes, 94);
    assert_eq!(summary.health.torn_tail_bytes, 4);
    assert_eq!(summary.health.rebuilt_index_topics, 1);
    assert_eq!(summary.health.unhealthy_cursors, 2);
    assert_eq!(summary.health.missing_configured_topics, 0);
    assert!(!summary.recovery.recovery_ready);
    assert_eq!(summary.recovery.cursor_blockers.len(), 2);
    assert_eq!(summary.recovery.cursor_blockers[0].status, "ahead_of_topic");
    assert_eq!(summary.recovery.cursor_blockers[1].status, "missing_topic");
    assert_eq!(summary.topics[0].last_valid_offset, Some(2));
    assert!(summary.topics[0].replayable);
    assert_eq!(
        summary.topics[0].transaction_locate_command.as_deref(),
        Some(
            "trellara stream locate-local --config <config> --transaction-id <tx> --commit-lsn <lsn> --topic trellara.local-source.retail-sales.strict"
        )
    );
    assert_eq!(
        summary.topics[0].replay_last_valid_command.as_deref(),
        Some(
            "trellara stream seek-local --config <config> --topic trellara.local-source.retail-sales.strict --next-offset 2"
        )
    );
    assert_eq!(summary.topics[0].torn_tail_bytes, 4);
    assert_eq!(summary.topics[0].index_entries, 3);
    assert_eq!(summary.topics[0].index_bytes, 24);
    assert_eq!(summary.topics[0].index_status, "stale_rebuilt");
    assert_eq!(summary.topics[0].last_proof_headers.len(), 1);
    assert_eq!(
        summary.topics[0].last_proof_headers[0].key,
        "trellara.ddl_target_ack_required"
    );
    assert_eq!(summary.cursors[0].topic_message_count, Some(3));
    assert_eq!(summary.cursors[0].pending_messages, Some(1));
    assert_eq!(summary.cursors[0].cursor_status, "ok");
    assert_eq!(summary.cursors[1].topic_message_count, Some(3));
    assert_eq!(summary.cursors[1].pending_messages, Some(0));
    assert_eq!(summary.cursors[1].cursor_status, "ahead_of_topic");
    assert_eq!(summary.cursors[2].topic_message_count, None);
    assert_eq!(summary.cursors[2].pending_messages, None);
    assert_eq!(summary.cursors[2].cursor_status, "missing_topic");
}
