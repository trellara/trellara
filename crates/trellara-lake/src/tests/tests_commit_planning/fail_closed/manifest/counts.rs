use super::*;

#[test]
fn partition_manifest_event_count_mismatch_fails_closed() {
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
        global_event_count: 2,
        partitions: Vec::new(),
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("mismatch");

    assert!(matches!(
        error,
        LakeError::ManifestEventCountMismatch {
            expected: 2,
            actual: 1
        }
    ));
}

#[test]
fn partition_manifest_event_count_sum_overflow_fails_closed() {
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
        partitions: vec![
            ManifestPartition {
                id: 0,
                event_count: u32::MAX,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 99,
            },
            ManifestPartition {
                id: 1,
                event_count: 1,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 100,
            },
        ],
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("overflow");

    assert!(matches!(
        error,
        LakeError::ManifestCountOverflow {
            field: "partition.event_count",
            max_supported_count: u32::MAX,
        }
    ));
}

#[test]
fn partition_manifest_affected_table_count_mismatch_fails_closed() {
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
        partitions: vec![ManifestPartition {
            id: 0,
            event_count: 2,
            first_total_order: 1,
            last_total_order: 2,
            checksum: 99,
        }],
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("affected count mismatch");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(
            trellara_protocol::ProtocolError::ManifestAffectedTableCountMismatch {
                transaction_id,
                expected: 2,
                actual: 1,
            }
        ) if transaction_id == "tx-1"
    ));
}
