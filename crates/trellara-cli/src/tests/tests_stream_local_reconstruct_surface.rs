use super::*;

#[test]
fn local_stream_reconstruct_requires_exact_commit_lsn_boundary() {
    let error = validate_local_stream_reconstruct_boundary(&LocalStreamReconstructArgs {
        config: PathBuf::from("local.yml"),
        transaction_id: "tx-partitioned-local".to_string(),
        commit_lsn: None,
    })
    .expect_err("missing commit lsn rejected");

    assert!(error
        .to_string()
        .contains("commit_lsn is required for exact transaction reconstruction"));
}

#[test]
fn local_stream_reconstruct_rejects_invalid_commit_lsn_boundary() {
    let error = validate_local_stream_reconstruct_boundary(&LocalStreamReconstructArgs {
        config: PathBuf::from("local.yml"),
        transaction_id: "tx-partitioned-local".to_string(),
        commit_lsn: Some("0/0".to_string()),
    })
    .expect_err("zero commit lsn rejected");

    assert!(error.to_string().contains("stream.locate.commit_lsn"));
    assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
}

#[tokio::test]
async fn local_stream_reconstruct_reports_source_order_proof() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-reconstruct-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let config = partitioned_local_config(&root);
    publish_local_partitioned_transaction(&config, "tx-partitioned-local", false).await;
    let summary = reconstruct_configured_local_stream_transaction(
        &config,
        &LocalStreamReconstructArgs {
            config: PathBuf::from("local.yml"),
            transaction_id: "tx-partitioned-local".to_string(),
            commit_lsn: Some("0/16B6C90".to_string()),
        },
    )
    .expect("reconstruct");

    assert!(summary.boundary.complete);
    assert_eq!(summary.boundary.mode, "partitioned_scale_mode");
    assert_eq!(summary.transaction_id, "tx-partitioned-local");
    assert_eq!(
        summary.partition_chunk_count,
        summary.partition_offsets.len()
    );
    assert_eq!(summary.reconstructed_change_count, 2);
    assert_eq!(summary.source_order, vec![1, 2]);
    assert!(summary
        .proof
        .contains("every partition chunk reconstructed before acknowledgement"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_stream_reconstruct_rejects_incomplete_partitioned_boundary() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-reconstruct-missing-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let config = partitioned_local_config(&root);
    publish_local_partitioned_transaction(&config, "tx-partitioned-local", true).await;
    let error = reconstruct_configured_local_stream_transaction(
        &config,
        &LocalStreamReconstructArgs {
            config: PathBuf::from("local.yml"),
            transaction_id: "tx-partitioned-local".to_string(),
            commit_lsn: Some("0/16B6C90".to_string()),
        },
    )
    .expect_err("incomplete boundary");

    assert!(error
        .to_string()
        .contains("cannot reconstruct incomplete local stream boundary"));
    assert!(error.to_string().contains("missing partition ids: 0"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_stream_reconstruct_reports_metadata_conflict_details() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-reconstruct-conflict-{}",
        unique_test_suffix()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let config = partitioned_local_config(&root);
    publish_local_partitioned_transaction_with_conflicting_source_id(
        &config,
        "tx-partitioned-local",
    )
    .await;
    let error = reconstruct_configured_local_stream_transaction(
        &config,
        &LocalStreamReconstructArgs {
            config: PathBuf::from("local.yml"),
            transaction_id: "tx-partitioned-local".to_string(),
            commit_lsn: Some("0/16B6C90".to_string()),
        },
    )
    .expect_err("metadata conflict");

    let message = error.to_string();
    assert!(message.contains("cannot reconstruct incomplete local stream boundary"));
    assert!(message.contains("metadata conflicts"));
    assert!(message.contains("boundary source IDs disagree: local-source, wrong-source"));

    std::fs::remove_dir_all(root).expect("cleanup");
}

fn partitioned_local_config(root: &Path) -> TrellaraConfig {
    let yaml = local_stream_yaml()
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 2\n    key_column: store_id",
        )
        .replace(
            "  path: /tmp/trellara-local-stream",
            &format!("  path: {}", root.display()),
        );
    TrellaraConfig::from_yaml(&yaml, "test").expect("parse")
}
