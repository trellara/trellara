use super::*;

#[tokio::test]
async fn local_stream_locate_reports_partitioned_metadata_conflicts() {
    let fixture = partitioned_fixture("trellara-cli-local-locate-partitioned-conflict");
    publish_local_partitioned_transaction_with_conflicting_commit_count(
        &fixture.config,
        "tx-partitioned-local",
    )
    .await;
    let args = partitioned_locate_args("tx-partitioned-local");

    let summary = locate_configured_local_stream_transaction(&fixture.config, &args)
        .expect("locate transaction");

    assert_eq!(summary.boundary.mode, "partitioned_scale_mode");
    assert!(!summary.boundary.complete);
    assert_eq!(summary.boundary.status, "incomplete_boundary");
    assert_eq!(summary.boundary.missing_message_kinds, Vec::<String>::new());
    assert_eq!(
        summary.boundary.metadata_conflicts,
        vec!["manifest and commit_marker partition counts disagree: 1, 2".to_string()]
    );
}

#[tokio::test]
async fn local_stream_locate_reports_partitioned_source_id_conflicts() {
    let fixture = partitioned_fixture("trellara-cli-local-locate-partitioned-source-conflict");
    publish_local_partitioned_transaction_with_conflicting_source_id(
        &fixture.config,
        "tx-partitioned-local",
    )
    .await;
    let args = partitioned_locate_args("tx-partitioned-local");

    let summary = locate_configured_local_stream_transaction(&fixture.config, &args)
        .expect("locate transaction");

    assert_eq!(summary.boundary.mode, "partitioned_scale_mode");
    assert!(!summary.boundary.complete);
    assert_eq!(summary.boundary.status, "incomplete_boundary");
    assert_eq!(
        summary.boundary.metadata_conflicts,
        vec!["boundary source IDs disagree: local-source, wrong-source".to_string()]
    );
}
