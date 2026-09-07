use super::*;

#[tokio::test]
async fn local_stream_locate_reports_complete_partitioned_boundary() {
    let fixture = partitioned_fixture("trellara-cli-local-locate-partitioned");
    publish_local_partitioned_transaction(&fixture.config, "tx-partitioned-local", false).await;
    let args = partitioned_locate_args("tx-partitioned-local");

    let summary = locate_configured_local_stream_transaction(&fixture.config, &args)
        .expect("locate transaction");

    assert_eq!(summary.boundary.mode, "partitioned_scale_mode");
    assert!(summary.boundary.complete);
    assert_eq!(summary.boundary.status, "complete");
    assert_eq!(
        summary.boundary.required_message_kinds,
        vec!["manifest", "commit_marker", "partition_chunk"]
    );
    assert_eq!(summary.boundary.missing_message_kinds, Vec::<String>::new());
    assert_eq!(
        summary.boundary.participating_partition_count,
        Some(summary.boundary.found_partition_count)
    );
    assert_eq!(summary.boundary.found_partition_ids, vec![0]);
    assert_eq!(summary.boundary.missing_partition_ids, Vec::<u32>::new());
    assert_eq!(summary.boundary.missing_partition_count, Some(0));
    assert!(summary
        .matches
        .iter()
        .any(|matched| matched.message_kind == "manifest"));
    assert!(summary
        .matches
        .iter()
        .any(|matched| matched.message_kind == "commit_marker"));
    assert!(
        summary
            .matches
            .iter()
            .any(|matched| matched.message_kind == "partition_chunk"
                && matched.partition_id.is_some())
    );
}

#[tokio::test]
async fn local_stream_locate_reports_incomplete_partitioned_boundary() {
    let fixture = partitioned_fixture("trellara-cli-local-locate-partitioned-missing");
    publish_local_partitioned_transaction(&fixture.config, "tx-partitioned-local", true).await;
    let args = partitioned_locate_args("tx-partitioned-local");

    let summary = locate_configured_local_stream_transaction(&fixture.config, &args)
        .expect("locate transaction");

    assert_eq!(summary.boundary.mode, "partitioned_scale_mode");
    assert!(!summary.boundary.complete);
    assert_eq!(summary.boundary.status, "incomplete_boundary");
    assert_eq!(
        summary.boundary.missing_message_kinds,
        vec!["partition_chunk".to_string()]
    );
    assert_eq!(
        summary.boundary.missing_partition_count,
        Some(
            summary
                .boundary
                .participating_partition_count
                .expect("expected partitions")
                - summary.boundary.found_partition_count
        )
    );
    assert_eq!(summary.boundary.found_partition_ids, Vec::<u32>::new());
    assert_eq!(summary.boundary.missing_partition_ids, vec![0]);
    assert!(summary.boundary.missing_partition_count.unwrap_or_default() > 0);
}
