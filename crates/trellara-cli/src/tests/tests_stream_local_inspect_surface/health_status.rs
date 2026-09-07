use super::*;

#[test]
fn local_stream_inspect_summary_reports_clean_health() {
    let summary = LocalStreamInspectSummary::from_inspection(
        LocalStreamInspection {
            root: PathBuf::from("/tmp/trellara-local-stream"),
            topics: vec![LocalTopicInspection {
                topic: "trellara.local-source.retail-sales.strict".to_string(),
                message_count: 3,
                last_valid_offset: Some(2),
                last_proof_headers: Vec::new(),
                replayable: true,
                valid_bytes: 90,
                file_bytes: 90,
                torn_tail_bytes: 0,
                index_entries: 3,
                index_bytes: 24,
                index_status: trellara_stream_local::LocalTopicIndexStatus::Healthy,
            }],
            cursors: vec![LocalCursorInspection::from_topic_depth(
                "applier",
                "trellara.local-source.retail-sales.strict",
                1,
                Some(3),
            )],
        },
        vec!["trellara.local-source.retail-sales.strict".to_string()],
    )
    .expect("summary");

    assert_eq!(summary.health.status, "clean");
    assert_eq!(summary.health.topic_count, 1);
    assert_eq!(summary.health.cursor_count, 1);
    assert_eq!(summary.health.total_messages, 3);
    assert_eq!(summary.health.total_pending_messages, 2);
    assert_eq!(summary.health.torn_tail_bytes, 0);
    assert_eq!(summary.health.rebuilt_index_topics, 0);
    assert_eq!(summary.health.unhealthy_cursors, 0);
    assert_eq!(summary.health.missing_configured_topics, 0);
    assert!(summary.recovery.recovery_ready);
    assert!(summary.recovery.torn_tail_topics.is_empty());
    assert!(summary.recovery.recovered_index_topics.is_empty());
    assert!(summary.recovery.cursor_blockers.is_empty());
    assert!(summary.recovery.recovery_actions.is_empty());
}

#[test]
fn local_stream_inspect_summary_reports_recovered_health() {
    let summary = LocalStreamInspectSummary::from_inspection(
        LocalStreamInspection {
            root: PathBuf::from("/tmp/trellara-local-stream"),
            topics: vec![LocalTopicInspection {
                topic: "trellara.local-source.retail-sales.strict".to_string(),
                message_count: 3,
                last_valid_offset: Some(2),
                last_proof_headers: Vec::new(),
                replayable: true,
                valid_bytes: 90,
                file_bytes: 94,
                torn_tail_bytes: 4,
                index_entries: 3,
                index_bytes: 24,
                index_status: trellara_stream_local::LocalTopicIndexStatus::StaleRebuilt,
            }],
            cursors: vec![LocalCursorInspection::from_topic_depth(
                "applier",
                "trellara.local-source.retail-sales.strict",
                1,
                Some(3),
            )],
        },
        vec!["trellara.local-source.retail-sales.strict".to_string()],
    )
    .expect("summary");

    assert_eq!(summary.health.status, "recovered");
    assert_eq!(summary.health.torn_tail_bytes, 4);
    assert_eq!(summary.health.rebuilt_index_topics, 1);
    assert_eq!(summary.health.unhealthy_cursors, 0);
    assert_eq!(summary.health.missing_configured_topics, 0);
    assert!(!summary.recovery.recovery_ready);
    assert_eq!(summary.recovery.torn_tail_topics.len(), 1);
    assert_eq!(
        summary.recovery.torn_tail_topics[0].topic,
        "trellara.local-source.retail-sales.strict"
    );
    assert_eq!(summary.recovery.recovered_index_topics.len(), 1);
    assert_eq!(
        summary.recovery.recovered_index_topics[0].index_status,
        "stale_rebuilt"
    );
    assert_eq!(summary.recovery.recovery_actions.len(), 2);
    assert_eq!(
        summary.recovery.recovery_actions[0].code,
        "truncate_torn_tail_on_next_publish"
    );
    assert_eq!(
        summary.recovery.recovery_actions[0].command,
        "trellara relay --config <flow>"
    );
    assert_eq!(
        summary.recovery.recovery_actions[1].code,
        "index_rebuilt_from_segment_log"
    );
}
