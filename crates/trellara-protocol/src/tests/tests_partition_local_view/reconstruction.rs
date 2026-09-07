use super::*;

#[test]
fn barrier_reconstruction_restores_source_order() {
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

    let reconstructed =
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("reconstruct");

    assert_eq!(
        reconstructed
            .iter()
            .map(|change| change.total_order)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

#[test]
fn partition_local_reader_returns_only_one_lane() {
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

    let local = partition_local_changes(&plan, partition_id);

    assert_eq!(
        local
            .iter()
            .map(|change| change.total_order)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
}
