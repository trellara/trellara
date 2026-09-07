use super::*;

#[test]
fn partition_manifest_missing_first_order_boundary_fails_closed() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        2,
        None,
        Some(row("sale-2", "20", "2026-01-01")),
    )]);
    envelope.manifest = Some(trellara_protocol::TransactionManifest {
        transaction_id: "tx-1".to_string(),
        source_commit_lsn: "0/16B6C50".to_string(),
        source_commit_timestamp_ms: 1_700_000_000,
        global_event_count: 1,
        partitions: vec![ManifestPartition {
            id: 3,
            event_count: 1,
            first_total_order: 1,
            last_total_order: 2,
            checksum: 99,
        }],
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("missing first order");

    assert!(matches!(
        error,
        LakeError::ManifestOrderBoundaryMissing {
            partition_id: 3,
            field: "first_total_order",
            total_order: 1,
        }
    ));
}

#[test]
fn partition_manifest_missing_last_order_boundary_fails_closed() {
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
            id: 3,
            event_count: 1,
            first_total_order: 1,
            last_total_order: 2,
            checksum: 99,
        }],
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("missing last order");

    assert!(matches!(
        error,
        LakeError::ManifestOrderBoundaryMissing {
            partition_id: 3,
            field: "last_total_order",
            total_order: 2,
        }
    ));
}

#[test]
fn partition_manifest_unknown_boundary_mode_fails_closed() {
    for boundary_mode in [
        trellara_protocol::ManifestBoundaryMode::Unspecified as i32,
        99,
    ] {
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
            boundary_mode,
        });
        envelope.finalize_checksum();

        let error = plan_commit(&envelope, &config()).expect_err("invalid boundary mode");

        assert!(matches!(
            error,
            LakeError::InvalidManifestBoundaryMode { boundary_mode: actual } if actual == boundary_mode
        ));
    }
}
