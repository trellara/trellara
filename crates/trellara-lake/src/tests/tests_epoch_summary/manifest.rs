use super::*;

#[test]
fn incomplete_manifest_blocks_epoch_completion() {
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
            event_count: 1,
            first_total_order: 1,
            last_total_order: 1,
            checksum: 99,
        }],
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = build_epoch_summary(
        &epoch_config(&["source-a"], LakeStragglerPolicy::WaitAllRequired),
        &[envelope],
    )
    .expect_err("incomplete manifest");

    assert!(matches!(
        error,
        LakeError::ManifestEventCountMismatch {
            expected: 2,
            actual: 1
        }
    ));
}

#[test]
fn manifest_boundary_mismatch_blocks_epoch_completion() {
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

    let error = build_epoch_summary(
        &epoch_config(&["source-a"], LakeStragglerPolicy::WaitAllRequired),
        &[envelope],
    )
    .expect_err("manifest mismatch");

    assert!(matches!(
        error,
        LakeError::ManifestBoundaryMismatch {
            field: "transaction_id",
            ..
        }
    ));
}

#[test]
fn duplicate_transaction_event_order_blocks_epoch_completion() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    let mut duplicate = envelope.changes[0].clone();
    duplicate.idempotency_key = idempotency_key("source-a", "0/16B6C50", "tx-1", 99);
    envelope.changes.push(duplicate);
    envelope.finalize_checksum();

    let error = build_epoch_summary(
        &epoch_config(&["source-a"], LakeStragglerPolicy::WaitAllRequired),
        &[envelope],
    )
    .expect_err("duplicate event order");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(
            trellara_protocol::ProtocolError::DuplicateTransactionEventOrder { total_order: 1 }
        )
    ));
}
