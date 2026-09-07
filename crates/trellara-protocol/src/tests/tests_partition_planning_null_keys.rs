use super::*;

#[test]
fn partition_planning_routes_null_key_to_singleton_partition_when_configured() {
    let mut null_key_change = sale_change(1, "store-104");
    null_key_change.after = Some(RowImage::new(vec![ColumnValue::null(
        "store_id", 25, false,
    )]));
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![null_key_change, sale_change(2, "store-104")],
    });

    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::RouteToSingletonPartition,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");

    let singleton = plan
        .chunks
        .iter()
        .find(|chunk| chunk.partition_id == 0)
        .expect("singleton null-key chunk");
    assert_eq!(singleton.changes[0].total_order, 1);
    assert!(plan
        .manifest
        .partitions
        .iter()
        .any(|partition| partition.id == 0));
    reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("barrier");
}

#[test]
fn partition_planning_routes_null_key_to_dead_letter_partition_when_configured() {
    let mut change = sale_change(1, "store-104");
    change.after = Some(RowImage::new(vec![ColumnValue::null(
        "store_id", 25, false,
    )]));
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![change, sale_change(2, "store-104")],
    });

    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::RouteToDeadLetterPartition,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");

    let dead_letter = plan
        .chunks
        .iter()
        .find(|chunk| chunk.partition_id == 7)
        .expect("dead-letter null-key chunk");
    assert_eq!(dead_letter.changes[0].total_order, 1);
    assert!(plan
        .manifest
        .partitions
        .iter()
        .any(|partition| partition.id == 7));
    reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("barrier");
}

#[test]
fn partition_planning_derives_null_key_partition_from_primary_key_when_configured() {
    let mut change = sale_change(1, "store-104");
    change.after = Some(RowImage::new(vec![
        ColumnValue::text("id", 23, "sale-null-owner", true),
        ColumnValue::null("store_id", 25, false),
    ]));
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![change],
    });
    let expected_partition = partition_for_key(
        &primary_key_bytes_from_row(envelope.changes[0].after.as_ref().expect("after"), 1)
            .expect("primary key route"),
        8,
    );

    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::DeriveFromPrimaryKey,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");

    assert_eq!(plan.chunks.len(), 1);
    assert_eq!(plan.chunks[0].partition_id, expected_partition);
    assert_eq!(plan.manifest.global_event_count, 1);
    reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("barrier");
}
