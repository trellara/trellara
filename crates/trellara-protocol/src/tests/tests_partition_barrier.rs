use super::*;

#[test]
fn partition_hash_is_stable() {
    assert_eq!(
        partition_for_key(b"store-104", 64),
        partition_for_key(b"store-104", 64)
    );
}

#[test]
fn partitioned_plan_emits_manifest_and_chunks() {
    let envelope = multi_store_envelope();
    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");

    assert_eq!(plan.manifest.transaction_id, "tx-1");
    assert_eq!(plan.manifest.source_commit_lsn, "0/16B6C50");
    assert_eq!(plan.manifest.global_event_count, 4);
    assert_eq!(
        ManifestBoundaryMode::try_from(plan.manifest.boundary_mode)
            .unwrap_or(ManifestBoundaryMode::Unspecified),
        ManifestBoundaryMode::PartitionedScale
    );
    assert_eq!(plan.manifest.partitions.len(), plan.chunks.len());
    assert_eq!(plan.manifest.affected_tables.len(), 1);
    assert_eq!(plan.manifest.affected_tables[0].event_count, 4);

    for partition in &plan.manifest.partitions {
        let chunk = plan
            .chunks
            .iter()
            .find(|chunk| chunk.partition_id == partition.id)
            .expect("chunk for manifest partition");
        assert_eq!(partition.event_count, chunk.changes.len() as u32);
        assert_eq!(partition.checksum, chunk.checksum);
        chunk.verify_checksum().expect("chunk checksum");
    }
}

#[test]
fn partitioned_plan_rejects_empty_transaction_boundary() {
    let error = plan_partitioned_transaction(
        &empty_envelope(),
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect_err("empty partitioned transaction");

    assert!(matches!(
        error,
        ProtocolError::EmptyTransaction { transaction_id } if transaction_id == "tx-empty"
    ));
}

#[test]
fn partitioned_plan_rejects_zero_partition_count() {
    let error = plan_partitioned_transaction(
        &multi_store_envelope(),
        &PartitionPlanConfig {
            partition_count: 0,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect_err("zero partition count");

    assert!(matches!(error, ProtocolError::InvalidPartitionCount));
}
