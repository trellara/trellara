use super::*;

#[test]
fn local_stream_locate_rejects_empty_transaction_id() {
    let error = validate_local_stream_locate_boundary(" ", Some("0/16B6C50"))
        .expect_err("empty transaction id rejected");

    assert!(error
        .to_string()
        .contains("transaction_id must not be empty"));
}

#[test]
fn local_stream_locate_rejects_transaction_id_with_surrounding_whitespace() {
    for transaction_id in [" tx-local-seek-1", "tx-local-seek-1 "] {
        let error = validate_local_stream_locate_boundary(transaction_id, Some("0/16B6C50"))
            .expect_err("spaced transaction id rejected");

        assert!(error
            .to_string()
            .contains("transaction_id must not contain surrounding whitespace"));
    }
}

#[test]
fn local_stream_locate_rejects_invalid_commit_lsn_filter() {
    for commit_lsn in [" ", "not-an-lsn", "0/0"] {
        let error = validate_local_stream_locate_boundary("tx-local-seek-1", Some(commit_lsn))
            .expect_err("invalid commit lsn rejected");

        assert!(error.to_string().contains("stream.locate.commit_lsn"));
    }
}

#[test]
fn local_stream_locate_rejects_commit_lsn_filter_with_surrounding_whitespace() {
    for commit_lsn in [" 0/16B6C50", "0/16B6C50 "] {
        let error = validate_local_stream_locate_boundary("tx-local-seek-1", Some(commit_lsn))
            .expect_err("spaced commit lsn rejected");

        assert!(error
            .to_string()
            .contains("commit_lsn must not contain surrounding whitespace"));
    }
}

#[tokio::test]
async fn local_stream_locate_finds_exact_transaction_seek_offset() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-locate-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_strict_transactions(&config, 3).await;
    let args = LocalStreamLocateArgs {
        config: PathBuf::from("local.yml"),
        transaction_id: "tx-local-seek-1".to_string(),
        commit_lsn: Some("0/16B6C51".to_string()),
        topic: None,
    };

    let summary =
        locate_configured_local_stream_transaction(&config, &args).expect("locate transaction");

    assert_eq!(summary.root, root.display().to_string());
    assert_eq!(summary.transaction_id, "tx-local-seek-1");
    assert_eq!(summary.commit_lsn.as_deref(), Some("0/16B6C51"));
    assert!(summary.exact_boundary);
    assert!(summary.replay_safe);
    assert!(summary.replay_warnings.is_empty());
    assert_eq!(
        summary.topics_scanned,
        vec!["trellara.local-source.retail-sales.strict".to_string()]
    );
    assert_eq!(summary.match_count, 1);
    assert_eq!(
        summary.matches[0].topic,
        "trellara.local-source.retail-sales.strict"
    );
    assert_eq!(summary.matches[0].offset, 1);
    assert_eq!(summary.matches[0].next_offset, 2);
    assert_eq!(summary.matches[0].message_kind, "strict_transaction");
    assert_eq!(
        summary.matches[0].source_id.as_deref(),
        Some("local-source")
    );
    assert_eq!(
        summary.matches[0].dataset_id.as_deref(),
        Some("retail-sales")
    );
    assert_eq!(summary.matches[0].commit_lsn.as_deref(), Some("0/16B6C51"));
    assert_eq!(
            summary.matches[0].seek_command,
            "trellara stream seek-local --config <config> --topic trellara.local-source.retail-sales.strict --next-offset 1 --transaction-id tx-local-seek-1 --commit-lsn 0/16B6C51"
        );
    assert_eq!(
        summary.next_commands,
        vec![summary.matches[0].seek_command.clone()]
    );

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_stream_locate_commit_lsn_filter_prevents_boundary_mismatch() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-locate-lsn-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_strict_transactions(&config, 2).await;
    let args = LocalStreamLocateArgs {
        config: PathBuf::from("local.yml"),
        transaction_id: "tx-local-seek-1".to_string(),
        commit_lsn: Some("0/16BFFFF".to_string()),
        topic: Some("trellara.local-source.retail-sales.strict".to_string()),
    };

    let summary =
        locate_configured_local_stream_transaction(&config, &args).expect("locate transaction");

    assert_eq!(summary.match_count, 0);
    assert!(summary.exact_boundary);
    assert!(!summary.replay_safe);
    assert!(summary
        .replay_warnings
        .contains(&"local boundary status is not_found".to_string()));
    assert!(summary
        .replay_warnings
        .contains(&"no matching local stream messages found".to_string()));
    assert!(summary.matches.is_empty());
    assert!(summary.next_commands.is_empty());

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_stream_locate_marks_transaction_id_only_lookup_not_replay_safe() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-locate-not-exact-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_strict_transactions(&config, 2).await;
    let args = LocalStreamLocateArgs {
        config: PathBuf::from("local.yml"),
        transaction_id: "tx-local-seek-1".to_string(),
        commit_lsn: None,
        topic: None,
    };

    let summary =
        locate_configured_local_stream_transaction(&config, &args).expect("locate transaction");

    assert_eq!(summary.match_count, 1);
    assert!(!summary.exact_boundary);
    assert!(!summary.replay_safe);
    assert!(summary.replay_warnings.iter().any(|warning| {
        warning.contains("commit_lsn is required")
            && warning.contains("transaction-boundary location")
    }));

    std::fs::remove_dir_all(root).expect("cleanup");
}
