use super::*;

#[test]
fn local_stream_seek_rejects_unconfigured_topics() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let args = LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic: "trellara.other.dataset.strict".to_string(),
        next_offset: 0,
        transaction_id: None,
        commit_lsn: None,
        consumer_group: None,
        allow_ahead: false,
    };

    assert!(matches!(
        seek_configured_local_stream(&config, &args),
        Err(CliError::InvalidConfig(message))
            if message.contains("is not configured for this flow")
    ));
}

#[test]
fn local_stream_seek_rejects_negative_offsets_before_stream_access() {
    let yaml = local_stream_yaml();
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let args = LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic: "trellara.local-source.retail-sales.strict".to_string(),
        next_offset: -1,
        transaction_id: None,
        commit_lsn: None,
        consumer_group: None,
        allow_ahead: false,
    };

    assert!(matches!(
        seek_configured_local_stream(&config, &args),
        Err(CliError::InvalidConfig(message))
            if message.contains("stream.seek.next_offset must be non-negative")
    ));
}

#[test]
fn local_stream_seek_sets_default_group_cursor() {
    let root = std::env::temp_dir().join(format!("trellara-cli-local-seek-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let args = LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic: "trellara.local-source.retail-sales.strict".to_string(),
        next_offset: 0,
        transaction_id: None,
        commit_lsn: None,
        consumer_group: None,
        allow_ahead: false,
    };

    let summary = seek_configured_local_stream(&config, &args).expect("seek");

    assert_eq!(summary.root, root.display().to_string());
    assert_eq!(
        summary.group_id,
        "trellara-applier-local-source-retail-sales"
    );
    assert_eq!(summary.previous_next_offset, None);
    assert_eq!(summary.next_offset, 0);
    assert_eq!(summary.topic_message_count, 0);
    assert_eq!(summary.pending_before, None);
    assert_eq!(summary.pending_after, 0);
    assert_eq!(summary.movement, "initialized");
    assert_eq!(summary.redelivered_messages, 0);
    assert_eq!(summary.skipped_messages, 0);
    assert_eq!(summary.cursor_status, "ok");
    assert!(!summary.allow_ahead);
    let inspection =
        inspect_local_stream(&root).expect("inspect local stream after seek operation");
    assert_eq!(inspection.cursors[0].next_offset, 0);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn local_stream_seek_rejects_accidental_fast_forward() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-seek-ahead-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let args = LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic: "trellara.local-source.retail-sales.strict".to_string(),
        next_offset: 7,
        transaction_id: None,
        commit_lsn: None,
        consumer_group: None,
        allow_ahead: false,
    };

    let error = seek_configured_local_stream(&config, &args).expect_err("reject seek ahead");

    assert!(matches!(
        error,
        CliError::LocalStream(trellara_stream_local::LocalStreamError::CursorOffsetAhead {
            offset: 7,
            message_count: 0,
            ..
        })
    ));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn local_stream_seek_allows_explicit_fast_forward() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-seek-allow-ahead-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let args = LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic: "trellara.local-source.retail-sales.strict".to_string(),
        next_offset: 7,
        transaction_id: None,
        commit_lsn: None,
        consumer_group: None,
        allow_ahead: true,
    };

    let summary = seek_configured_local_stream(&config, &args).expect("seek ahead");

    assert_eq!(summary.previous_next_offset, None);
    assert_eq!(summary.next_offset, 7);
    assert_eq!(summary.topic_message_count, 0);
    assert_eq!(summary.pending_before, None);
    assert_eq!(summary.pending_after, 0);
    assert_eq!(summary.movement, "initialized");
    assert_eq!(summary.redelivered_messages, 0);
    assert_eq!(summary.skipped_messages, 0);
    assert_eq!(summary.cursor_status, "ahead_of_topic");
    assert!(summary.allow_ahead);
    let inspection =
        inspect_local_stream(&root).expect("inspect local stream after seek operation");
    assert_eq!(inspection.cursors[0].next_offset, 7);

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn local_stream_seek_verifies_expected_transaction_before_cursor_write() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-seek-guard-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_strict_transactions(&config, 2).await;
    let args = LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic: "trellara.local-source.retail-sales.strict".to_string(),
        next_offset: 1,
        transaction_id: Some("tx-local-seek-0".to_string()),
        commit_lsn: None,
        consumer_group: None,
        allow_ahead: false,
    };

    let error = seek_configured_local_stream(&config, &args).expect_err("reject mismatch");

    assert!(matches!(
        error,
        CliError::InvalidConfig(message)
            if message.contains("stream.seek transaction_id mismatch")
                && message.contains("expected tx-local-seek-0")
                && message.contains("found tx-local-seek-1")
    ));
    let inspection = inspect_local_stream(&root).expect("inspect local stream after rejection");
    assert!(inspection.cursors.is_empty());

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn local_stream_seek_verifies_expected_commit_lsn_before_cursor_write() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-seek-lsn-guard-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_strict_transactions(&config, 2).await;
    let args = LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic: "trellara.local-source.retail-sales.strict".to_string(),
        next_offset: 1,
        transaction_id: Some("tx-local-seek-1".to_string()),
        commit_lsn: Some("0/16BFFFF".to_string()),
        consumer_group: None,
        allow_ahead: false,
    };

    let error = seek_configured_local_stream(&config, &args).expect_err("reject mismatch");

    assert!(matches!(
        error,
        CliError::InvalidConfig(message)
            if message.contains("stream.seek commit_lsn mismatch")
                && message.contains("expected 0/16BFFFF")
                && message.contains("found 0/16B6C51")
    ));
    let inspection = inspect_local_stream(&root).expect("inspect local stream after rejection");
    assert!(inspection.cursors.is_empty());

    let _ = std::fs::remove_dir_all(root);
}
