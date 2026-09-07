use super::*;

#[tokio::test]
async fn local_stream_seek_reports_incomplete_partitioned_boundary() {
    let fixture = partitioned_fixture("trellara-cli-local-seek-partitioned-boundary");
    publish_local_partitioned_transaction(&fixture.config, "tx-partitioned-local-seek", true).await;
    let seek = seek_args("trellara.local-source.retail-sales.manifest".to_string(), 0);

    let summary = seek_configured_local_stream(&fixture.config, &seek).expect("seek manifest");

    let boundary = summary.boundary.as_ref().expect("seek boundary");
    assert_eq!(boundary.mode, "partitioned_scale_mode");
    assert_eq!(boundary.status, "incomplete_boundary");
    assert!(!boundary.complete);
    assert!(boundary.missing_partition_count.unwrap_or_default() > 0);
    assert_eq!(boundary.missing_partition_ids, vec![0]);
    assert_eq!(
        summary.boundary_warnings,
        vec![
            "local boundary status is incomplete_boundary".to_string(),
            "missing message kinds: partition_chunk".to_string(),
            "missing partition ids: 0".to_string()
        ]
    );
    assert!(!summary.replay_safe);
    assert!(summary
        .replay_warnings
        .contains(&"local boundary status is incomplete_boundary".to_string()));
    assert!(summary
        .boundary_seek_commands
        .iter()
        .any(|command| command
            .contains("trellara.local-source.retail-sales.manifest --next-offset 0")));
    assert!(summary.boundary_seek_commands.iter().any(|command| {
        command.contains("trellara.local-source.retail-sales.commit --next-offset 0")
    }));
}

#[tokio::test]
async fn local_stream_seek_reports_boundary_metadata_warnings() {
    let fixture = partitioned_fixture("trellara-cli-local-seek-partitioned-conflict");
    publish_local_partitioned_transaction_with_conflicting_source_id(
        &fixture.config,
        "tx-partitioned-local-seek",
    )
    .await;
    let seek = seek_args("trellara.local-source.retail-sales.manifest".to_string(), 0);

    let summary = seek_configured_local_stream(&fixture.config, &seek).expect("seek manifest");

    assert_eq!(
        summary.boundary_warnings,
        vec![
            "local boundary status is incomplete_boundary".to_string(),
            "metadata conflicts: boundary source IDs disagree: local-source, wrong-source"
                .to_string()
        ]
    );
    assert!(!summary.replay_safe);
    assert!(summary
        .replay_warnings
        .contains(&"local boundary status is incomplete_boundary".to_string()));
}
