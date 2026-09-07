use super::*;

#[test]
fn epoch_summary_rolls_up_manifest_partition_evidence() {
    let mut first = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![
            change(
                Operation::Insert,
                1,
                None,
                Some(row("sale-1", "10", "2026-01-01")),
            ),
            change(
                Operation::Insert,
                2,
                None,
                Some(row("sale-2", "20", "2026-01-01")),
            ),
        ],
    );
    first.manifest = Some(partition_manifest(
        "tx-1",
        "0/16B6C50",
        vec![
            ManifestPartition {
                id: 0,
                event_count: 1,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 11,
            },
            ManifestPartition {
                id: 1,
                event_count: 1,
                first_total_order: 2,
                last_total_order: 2,
                checksum: 22,
            },
        ],
    ));
    first.finalize_checksum();

    let mut second = envelope_for(
        "store-001",
        "tx-2",
        "0/16B6D00",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-3", "30", "2026-01-01")),
        )],
    );
    second.manifest = Some(partition_manifest(
        "tx-2",
        "0/16B6D00",
        vec![ManifestPartition {
            id: 1,
            event_count: 1,
            first_total_order: 1,
            last_total_order: 1,
            checksum: 44,
        }],
    ));
    second.finalize_checksum();

    let summary = build_epoch_summary(
        &epoch_config(&["store-001"], LakeStragglerPolicy::WaitAllRequired),
        &[first, second],
    )
    .expect("epoch summary");

    assert_eq!(summary.partitions.len(), 2);
    assert_eq!(summary.partitions[0].source_id, "store-001");
    assert_eq!(summary.partitions[0].partition_id, 0);
    assert_eq!(
        summary.partitions[0].first_commit_lsn.as_deref(),
        Some("0/16B6C50")
    );
    assert_eq!(
        summary.partitions[0].last_commit_lsn.as_deref(),
        Some("0/16B6C50")
    );
    assert_eq!(summary.partitions[0].transaction_count, 1);
    assert_eq!(summary.partitions[0].event_count, 1);
    assert_eq!(summary.partitions[0].checksum_rollup, 11);

    assert_eq!(summary.partitions[1].source_id, "store-001");
    assert_eq!(summary.partitions[1].partition_id, 1);
    assert_eq!(
        summary.partitions[1].first_commit_lsn.as_deref(),
        Some("0/16B6C50")
    );
    assert_eq!(
        summary.partitions[1].last_commit_lsn.as_deref(),
        Some("0/16B6D00")
    );
    assert_eq!(summary.partitions[1].transaction_count, 2);
    assert_eq!(summary.partitions[1].event_count, 2);
    assert_eq!(summary.partitions[1].checksum_rollup, 22 ^ 44);
}

#[test]
fn epoch_summary_leaves_partition_evidence_empty_without_manifest() {
    let envelope = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "10", "2026-01-01")),
        )],
    );

    let summary = build_epoch_summary(
        &epoch_config(&["store-001"], LakeStragglerPolicy::WaitAllRequired),
        &[envelope],
    )
    .expect("epoch summary");

    assert!(summary.partitions.is_empty());
}

fn partition_manifest(
    transaction_id: &str,
    source_commit_lsn: &str,
    partitions: Vec<ManifestPartition>,
) -> trellara_protocol::TransactionManifest {
    let global_event_count = partitions
        .iter()
        .map(|partition| partition.event_count)
        .sum();
    trellara_protocol::TransactionManifest {
        transaction_id: transaction_id.to_string(),
        source_commit_lsn: source_commit_lsn.to_string(),
        source_commit_timestamp_ms: 1_700_000_000,
        global_event_count,
        partitions,
        affected_tables: affected_tables(global_event_count),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    }
}
