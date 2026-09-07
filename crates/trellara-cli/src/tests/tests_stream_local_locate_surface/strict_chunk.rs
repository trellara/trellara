use super::*;

#[tokio::test]
async fn local_stream_locate_reports_complete_strict_chunk_boundary() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-locate-strict-chunk-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml()
            .replace(
                "  tables:\n    - schema: public\n      name: sales",
                "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 1",
            )
            .replace(
                "  path: /tmp/trellara-local-stream",
                &format!("  path: {}", root.display()),
            );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    publish_local_strict_chunked_transaction(&config, "tx-strict-chunk-local").await;
    let args = LocalStreamLocateArgs {
        config: PathBuf::from("local.yml"),
        transaction_id: "tx-strict-chunk-local".to_string(),
        commit_lsn: Some("0/16B6D00".to_string()),
        topic: None,
    };

    let summary =
        locate_configured_local_stream_transaction(&config, &args).expect("locate transaction");

    assert_eq!(summary.boundary.mode, "strict_chunked_transaction_order");
    assert!(summary.boundary.complete);
    assert_eq!(summary.boundary.status, "complete");
    assert_eq!(
        summary.boundary.required_message_kinds,
        vec!["manifest", "commit_marker", "strict_chunk"]
    );
    assert_eq!(summary.boundary.missing_message_kinds, Vec::<String>::new());
    assert!(summary
        .matches
        .iter()
        .any(|matched| matched.message_kind == "strict_chunk" && matched.partition_id.is_some()));

    std::fs::remove_dir_all(root).expect("cleanup");
}
