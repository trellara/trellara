use super::*;

#[test]
fn partition_manifest_duplicate_partition_id_fails_closed() {
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
                event_count: 1,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 99,
            },
            ManifestPartition {
                id: 0,
                event_count: 0,
                first_total_order: 0,
                last_total_order: 0,
                checksum: 100,
            },
        ],
        affected_tables: affected_tables(1),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("duplicate partition");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(
            trellara_protocol::ProtocolError::DuplicateManifestPartition {
                transaction_id,
                partition_id: 0,
            }
        ) if transaction_id == "tx-1"
    ));
}

#[test]
fn partition_manifest_missing_affected_table_relation_fails_closed() {
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
        affected_tables: vec![AffectedTable {
            relation: None,
            event_count: 1,
        }],
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let error = plan_commit(&envelope, &config()).expect_err("missing affected relation");

    assert!(matches!(
        error,
        LakeError::InvalidEnvelope(
            trellara_protocol::ProtocolError::ManifestAffectedTableMissingRelation {
                transaction_id,
                index: 0,
            }
        ) if transaction_id == "tx-1"
    ));
}
