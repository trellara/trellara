use super::*;

#[tokio::test]
async fn local_stream_seek_reports_replay_boundary_delta() {
    let fixture = strict_fixture("trellara-cli-local-seek-replay");
    publish_local_strict_transactions(&fixture.config, 3).await;
    let topic = strict_topic();
    let initial = seek_args(topic.clone(), 3);
    seek_configured_local_stream(&fixture.config, &initial).expect("seek to end");
    let replay = seek_args(topic, 1);

    let summary = seek_configured_local_stream(&fixture.config, &replay).expect("seek for replay");

    assert_eq!(summary.previous_next_offset, Some(3));
    assert_eq!(summary.next_offset, 1);
    assert_eq!(summary.topic_message_count, 3);
    assert_eq!(summary.pending_before, Some(0));
    assert_eq!(summary.pending_after, 2);
    assert_eq!(summary.movement, "rewind_replay");
    assert_eq!(summary.redelivered_messages, 2);
    assert_eq!(summary.skipped_messages, 0);
    assert_eq!(summary.cursor_status, "ok");
    assert_eq!(
        summary.boundary.as_ref().expect("seek boundary").status,
        "complete"
    );
    assert_eq!(summary.boundary_warnings, Vec::<String>::new());
    assert!(!summary.replay_safe);
    assert_eq!(
        summary.replay_warnings,
        vec![
            "transaction_id is required for replay-safe local rewind".to_string(),
            "commit_lsn is required for replay-safe exact local rewind".to_string()
        ]
    );
    assert_eq!(
        summary.boundary_seek_commands,
        vec![
            "trellara stream seek-local --config <config> --topic trellara.local-source.retail-sales.strict --next-offset 1 --transaction-id tx-local-seek-1 --commit-lsn 0/16B6C51"
                .to_string()
        ]
    );
}

#[tokio::test]
async fn local_stream_seek_marks_exact_rewind_replay_safe() {
    let fixture = strict_fixture("trellara-cli-local-seek-exact-replay");
    publish_local_strict_transactions(&fixture.config, 3).await;
    let topic = strict_topic();
    let initial = seek_args(topic.clone(), 3);
    seek_configured_local_stream(&fixture.config, &initial).expect("seek to end");
    let replay = exact_seek_args(topic, 1, "tx-local-seek-1", "0/16B6C51");

    let summary = seek_configured_local_stream(&fixture.config, &replay).expect("seek for replay");

    assert_eq!(summary.movement, "rewind_replay");
    assert!(summary.boundary.as_ref().expect("seek boundary").complete);
    assert!(summary.replay_safe);
    assert!(summary.replay_warnings.is_empty());
}

#[tokio::test]
async fn local_stream_seek_reports_skip_boundary_delta() {
    let fixture = strict_fixture("trellara-cli-local-seek-skip");
    publish_local_strict_transactions(&fixture.config, 3).await;
    let topic = strict_topic();
    let initial = seek_args(topic.clone(), 1);
    seek_configured_local_stream(&fixture.config, &initial).expect("seek to first pending message");
    let skip = seek_args(topic, 3);

    let summary = seek_configured_local_stream(&fixture.config, &skip).expect("seek forward");

    assert_eq!(summary.previous_next_offset, Some(1));
    assert_eq!(summary.next_offset, 3);
    assert_eq!(summary.topic_message_count, 3);
    assert_eq!(summary.pending_before, Some(2));
    assert_eq!(summary.pending_after, 0);
    assert_eq!(summary.movement, "fast_forward_skip");
    assert_eq!(summary.redelivered_messages, 0);
    assert_eq!(summary.skipped_messages, 2);
    assert_eq!(summary.cursor_status, "ok");
    assert!(summary.replay_safe);
    assert!(summary.replay_warnings.is_empty());
}
