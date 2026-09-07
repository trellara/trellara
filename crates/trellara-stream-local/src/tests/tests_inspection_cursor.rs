use super::*;

#[test]
fn inspect_empty_stream_reports_no_topics_or_cursors() {
    let root = temp_root("inspect-empty");
    let inspection = inspect_local_stream(&root).expect("inspect");

    assert_eq!(inspection.root, root);
    assert!(inspection.topics.is_empty());
    assert!(inspection.cursors.is_empty());
}

#[tokio::test]
async fn inspect_stream_reports_topic_depth_and_cursor_offsets() {
    let root = temp_root("inspect");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-1", "one"))
        .await
        .expect("publish first");
    publisher
        .publish(message("trellara.source.dataset.strict", "tx-2", "two"))
        .await
        .expect("publish second");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec!["trellara.source.dataset.strict".to_string()],
    ))
    .expect("consumer");
    let first = consumer.next().await.expect("next").expect("message");
    consumer.ack(&first).await.expect("ack");

    let inspection = inspect_local_stream(&root).expect("inspect");

    assert_eq!(inspection.topics.len(), 1);
    assert_eq!(inspection.topics[0].topic, "trellara.source.dataset.strict");
    assert_eq!(inspection.topics[0].message_count, 2);
    assert_eq!(inspection.topics[0].last_valid_offset, Some(1));
    assert!(inspection.topics[0].replayable);
    assert_eq!(
        inspection.topics[0].valid_bytes,
        inspection.topics[0].file_bytes
    );
    assert_eq!(inspection.topics[0].index_entries, 2);
    assert_eq!(inspection.topics[0].index_bytes, 16);
    assert_eq!(
        inspection.topics[0].index_status,
        LocalTopicIndexStatus::Healthy
    );
    assert_eq!(
        inspection.cursors,
        vec![LocalCursorInspection::from_topic_depth(
            "applier",
            "trellara.source.dataset.strict",
            1,
            Some(2),
        )]
    );

    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn inspect_stream_reports_last_message_ddl_proof_headers() {
    let root = temp_root("inspect-proof-headers");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(StreamMessage {
            topic: "trellara.source.dataset.strict".to_string(),
            key: "source:dataset:tx-ddl:0/16B6C50".to_string(),
            payload: Bytes::from_static(b"ddl-envelope"),
            headers: vec![
                StreamHeader::new("trellara.commit_lsn", "0/16B6C50"),
                StreamHeader::new("trellara.ddl_event_count", "1"),
                StreamHeader::new("trellara.ddl_release_gates", "post_ddl_dml_release"),
                StreamHeader::new(
                    "trellara.ddl_propagation_decisions",
                    "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1",
                ),
                StreamHeader::new("trellara.ddl_target_ack_required", "1"),
                StreamHeader::new(
                    "trellara.ddl_propagation_policy_sha256",
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                ),
                StreamHeader::new("trellara.unrelated", "ignored"),
            ],
            partition: None,
            position: None,
        })
        .await
        .expect("publish DDL proof message");

    let inspection = inspect_local_stream(&root).expect("inspect");
    let proof_headers = &inspection.topics[0].last_proof_headers;

    assert_eq!(proof_headers.len(), 5);
    assert!(proof_headers.contains(&LocalTopicProofHeader {
        key: "trellara.commit_lsn".to_string(),
        value: "0/16B6C50".to_string(),
    }));
    assert!(proof_headers.contains(&LocalTopicProofHeader {
        key: "trellara.ddl_target_ack_required".to_string(),
        value: "1".to_string(),
    }));
    assert!(!proof_headers
        .iter()
        .any(|header| header.key == "trellara.unrelated"));

    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn inspect_stream_reports_missing_topic_cursor_status() {
    let root = temp_root("inspect-missing-topic-cursor");
    set_local_cursor_with_policy(&root, "applier", "trellara.source.dataset.strict", 4, true)
        .expect("write cursor");

    let inspection = inspect_local_stream(&root).expect("inspect");

    assert!(inspection.topics.is_empty());
    assert_eq!(
        inspection.cursors,
        vec![LocalCursorInspection::from_topic_depth(
            "applier",
            "trellara.source.dataset.strict",
            4,
            None,
        )]
    );
    assert_eq!(
        inspection.cursors[0].status,
        LocalCursorStatus::MissingTopic
    );
    assert_eq!(inspection.cursors[0].pending_messages, None);

    fs::remove_dir_all(root).expect("cleanup");
}
