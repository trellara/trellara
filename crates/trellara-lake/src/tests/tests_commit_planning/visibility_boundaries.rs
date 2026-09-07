use super::*;

#[test]
fn partition_manifest_is_the_visibility_boundary_when_present() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    envelope.manifest = Some(trellara_protocol::TransactionManifest {
        transaction_id: "tx-1".to_string(),
        source_commit_lsn: "0/16B6C50".to_string(),
        source_commit_timestamp_ms: 1_700_000_000,
        global_event_count: 1,
        partitions: vec![ManifestPartition {
            id: 0,
            event_count: 1,
            first_total_order: 1,
            last_total_order: 1,
            checksum: 99,
        }],
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let plan = plan_commit(&envelope, &config()).expect("plan");

    assert_eq!(
        plan.visibility_boundary,
        LakeVisibilityBoundary::PartitionManifestBarrier {
            global_event_count: 1,
            participating_partition_count: 1
        }
    );
}

#[test]
fn strict_chunk_manifest_is_the_visibility_boundary_when_present() {
    let mut envelope = envelope(vec![
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
    ]);
    envelope.manifest = Some(trellara_protocol::TransactionManifest {
        transaction_id: "tx-1".to_string(),
        source_commit_lsn: "0/16B6C50".to_string(),
        source_commit_timestamp_ms: 1_700_000_000,
        global_event_count: 2,
        partitions: vec![
            ManifestPartition {
                id: 0,
                event_count: 1,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 99,
            },
            ManifestPartition {
                id: 1,
                event_count: 1,
                first_total_order: 2,
                last_total_order: 2,
                checksum: 100,
            },
        ],
        affected_tables: affected_tables(2),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::StrictChunkedTransactionOrder
            as i32,
    });
    envelope.finalize_checksum();

    let plan = plan_commit(&envelope, &config()).expect("plan");

    assert_eq!(
        plan.visibility_boundary,
        LakeVisibilityBoundary::StrictChunkManifestBarrier {
            global_event_count: 2,
            chunk_count: 2
        }
    );
}
