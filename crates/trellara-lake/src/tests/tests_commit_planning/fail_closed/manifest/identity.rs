use super::*;

#[test]
fn partition_manifest_transaction_mismatch_fails_closed() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    envelope.manifest = Some(trellara_protocol::TransactionManifest {
        transaction_id: "tx-other".to_string(),
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

    let error = plan_commit(&envelope, &config()).expect_err("transaction mismatch");

    assert!(matches!(
        error,
        LakeError::ManifestBoundaryMismatch {
            field: "transaction_id",
            envelope,
            manifest,
        } if envelope == "tx-1" && manifest == "tx-other"
    ));
}

#[test]
fn partition_manifest_commit_lsn_mismatch_fails_closed() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    envelope.manifest = Some(trellara_protocol::TransactionManifest {
        transaction_id: "tx-1".to_string(),
        source_commit_lsn: "0/16B6C51".to_string(),
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

    let error = plan_commit(&envelope, &config()).expect_err("commit lsn mismatch");

    assert!(matches!(
        error,
        LakeError::ManifestBoundaryMismatch {
            field: "commit_lsn",
            envelope,
            manifest,
        } if envelope == "0/16B6C50" && manifest == "0/16B6C51"
    ));
}
