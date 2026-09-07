use super::*;

#[test]
fn local_stream_inspect_summary_reports_barrier_topic_presence() {
    let summary = LocalStreamInspectSummary::from_inspection(
        LocalStreamInspection {
            root: PathBuf::from("/tmp/trellara-local-stream"),
            topics: vec![
                LocalTopicInspection {
                    topic: "trellara.local-source.retail-sales.manifest".to_string(),
                    message_count: 1,
                    last_valid_offset: Some(0),
                    last_proof_headers: Vec::new(),
                    replayable: true,
                    valid_bytes: 90,
                    file_bytes: 90,
                    torn_tail_bytes: 0,
                    index_entries: 1,
                    index_bytes: 8,
                    index_status: trellara_stream_local::LocalTopicIndexStatus::Healthy,
                },
                LocalTopicInspection {
                    topic: "trellara.local-source.retail-sales.partition.0".to_string(),
                    message_count: 3,
                    last_valid_offset: Some(2),
                    last_proof_headers: Vec::new(),
                    replayable: true,
                    valid_bytes: 180,
                    file_bytes: 180,
                    torn_tail_bytes: 0,
                    index_entries: 3,
                    index_bytes: 24,
                    index_status: trellara_stream_local::LocalTopicIndexStatus::Healthy,
                },
            ],
            cursors: Vec::new(),
        },
        vec![
            "trellara.local-source.retail-sales.manifest".to_string(),
            "trellara.local-source.retail-sales.commit".to_string(),
            "trellara.local-source.retail-sales.partition.0".to_string(),
            "trellara.local-source.retail-sales.partition.1".to_string(),
        ],
    )
    .expect("summary");

    assert!(summary.barrier_topics.required);
    assert_eq!(summary.barrier_topics.expected_topic_count, 4);
    assert_eq!(summary.barrier_topics.observed_configured_topic_count, 2);
    assert_eq!(summary.barrier_topics.manifest_topic_present, Some(true));
    assert_eq!(summary.barrier_topics.commit_topic_present, Some(false));
    assert_eq!(
        summary.barrier_topics.missing_topics,
        vec![
            "trellara.local-source.retail-sales.commit".to_string(),
            "trellara.local-source.retail-sales.partition.1".to_string(),
        ]
    );
    assert_eq!(summary.health.status, "degraded");
    assert_eq!(summary.health.missing_configured_topics, 2);
}
