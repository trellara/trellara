use super::*;

#[test]
fn partition_local_view_carries_incomplete_global_metadata() {
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
    let partition_id = partition_for_key(b"store-104", 8);
    let chunk = plan
        .chunks
        .iter()
        .find(|chunk| chunk.partition_id == partition_id)
        .expect("store-104 chunk");

    let local = partition_local_view(&plan.manifest, chunk).expect("local view");
    let barrier =
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("barrier view");

    assert_eq!(local.transaction_id, "tx-1");
    assert_eq!(local.partition_id, partition_id);
    assert_eq!(local.partition_event_count, 2);
    assert_eq!(local.global_event_count, 4);
    assert_eq!(
        local.participating_partition_count,
        plan.chunks.len() as u32
    );
    assert!(!local.transaction_complete);
    assert_eq!(
        local
            .changes
            .iter()
            .map(|change| change.total_order)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    assert_eq!(
        barrier
            .iter()
            .map(|change| change.total_order)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

#[test]
fn partition_local_view_is_complete_for_single_partition_transaction() {
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-single".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![
            transaction_sale_change("tx-single", 1, "store-104"),
            transaction_sale_change("tx-single", 2, "store-104"),
        ],
    });
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
    assert_eq!(plan.chunks.len(), 1);

    let local = partition_local_view(&plan.manifest, &plan.chunks[0]).expect("local view");

    assert!(local.transaction_complete);
    assert_eq!(local.partition_event_count, 2);
    assert_eq!(local.global_event_count, 2);
    assert_eq!(local.first_total_order, 1);
    assert_eq!(local.last_total_order, 2);
}
