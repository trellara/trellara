use super::*;

#[test]
fn partition_local_view_rejects_chunk_missing_from_manifest() {
    let envelope = multi_store_envelope();
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    let chunk = plan.chunks.pop().expect("chunk");
    plan.manifest
        .partitions
        .retain(|partition| partition.id != chunk.partition_id);

    assert!(matches!(
        partition_local_view(&plan.manifest, &chunk),
        Err(ProtocolError::PartitionNotInManifest { partition_id, .. })
            if partition_id == chunk.partition_id
    ));
}

#[test]
fn partition_local_view_rejects_invalid_manifest_identity_fields() {
    for field in ["transaction_id", "source_commit_lsn", "global_event_count"] {
        let envelope = multi_store_envelope();
        let mut plan = plan_partitioned_transaction(
            &envelope,
            &PartitionPlanConfig {
                partition_count: 8,
                key_column: "store_id".to_string(),
                null_key_policy: PartitionNullKeyPolicy::Quarantine,
                key_change_policy: PartitionKeyChangePolicy::Quarantine,
            },
        )
        .expect("partition plan");

        match field {
            "transaction_id" => {
                plan.manifest.transaction_id = " tx-1".to_string();
                for chunk in &mut plan.chunks {
                    chunk.transaction_id = plan.manifest.transaction_id.clone();
                    for change in &mut chunk.changes {
                        change.transaction_id = plan.manifest.transaction_id.clone();
                    }
                    chunk.finalize_checksum();
                }
                for (partition, chunk) in plan.manifest.partitions.iter_mut().zip(&plan.chunks) {
                    partition.checksum = chunk.checksum;
                }
            }
            "source_commit_lsn" => plan.manifest.source_commit_lsn = "0/16B6C50 ".to_string(),
            "global_event_count" => plan.manifest.global_event_count = 0,
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            partition_local_view(&plan.manifest, &plan.chunks[0]),
            Err(ProtocolError::InvalidManifestField { field: actual, .. })
                if actual == field
        ));
    }
}

#[test]
fn partition_local_view_rejects_unsupported_change_operation() {
    let envelope = multi_store_envelope();
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    plan.chunks[0].changes[0].operation = 99;
    plan.chunks[0].finalize_checksum();
    plan.manifest.partitions[0].checksum = plan.chunks[0].checksum;
    let total_order = plan.chunks[0].changes[0].total_order;

    assert!(matches!(
        partition_local_view(&plan.manifest, &plan.chunks[0]),
        Err(ProtocolError::UnsupportedOperation {
            total_order: actual,
            operation: 99,
        }) if actual == total_order
    ));
}

#[test]
fn partition_local_view_rejects_unspecified_change_operation() {
    let envelope = multi_store_envelope();
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    plan.chunks[0].changes[0].operation = Operation::Unspecified as i32;
    plan.chunks[0].finalize_checksum();
    plan.manifest.partitions[0].checksum = plan.chunks[0].checksum;
    let total_order = plan.chunks[0].changes[0].total_order;

    assert!(matches!(
        partition_local_view(&plan.manifest, &plan.chunks[0]),
        Err(ProtocolError::UnsupportedOperation {
            total_order: actual,
            operation: 0,
        }) if actual == total_order
    ));
}
