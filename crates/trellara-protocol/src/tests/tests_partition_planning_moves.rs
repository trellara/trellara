use super::*;

#[test]
fn partition_planning_emits_move_as_delete_and_insert_under_one_manifest() {
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![ownership_move_change(1, "store-104", "store-205")],
    });
    let old_partition = partition_for_key(b"store-104", 8);
    let new_partition = partition_for_key(b"store-205", 8);

    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::EmitMove,
        },
    )
    .expect("partition plan");

    assert_eq!(plan.manifest.transaction_id, "tx-1");
    assert_eq!(plan.manifest.global_event_count, 2);
    assert_eq!(plan.manifest.affected_tables[0].event_count, 2);

    let delete = plan
        .chunks
        .iter()
        .flat_map(|chunk| {
            chunk
                .changes
                .iter()
                .map(move |change| (chunk.partition_id, change))
        })
        .find(|(_, change)| {
            Operation::try_from(change.operation).expect("operation") == Operation::Delete
        })
        .expect("delete side of move");
    let insert = plan
        .chunks
        .iter()
        .flat_map(|chunk| {
            chunk
                .changes
                .iter()
                .map(move |change| (chunk.partition_id, change))
        })
        .find(|(_, change)| {
            Operation::try_from(change.operation).expect("operation") == Operation::Insert
        })
        .expect("insert side of move");

    assert_eq!(delete.0, old_partition);
    assert_eq!(insert.0, new_partition);
    assert_eq!(
        delete.1.idempotency_key,
        "source-a:0/16B6C50:tx-1:1:move_delete"
    );
    assert_eq!(
        insert.1.idempotency_key,
        "source-a:0/16B6C50:tx-1:1:move_insert"
    );

    let reconstructed =
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks).expect("reconstruct");
    assert_eq!(
        reconstructed
            .iter()
            .map(|change| Operation::try_from(change.operation).expect("operation"))
            .collect::<Vec<_>>(),
        vec![Operation::Delete, Operation::Insert]
    );
}

#[test]
fn partition_planning_derives_move_with_null_key_side_from_primary_key() {
    let mut change = ownership_move_change(1, "store-104", "store-205");
    change.after = Some(RowImage::new(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::null("store_id", 25, false),
        ColumnValue::text("amount_cents", 20, "1299", false),
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

    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::DeriveFromPrimaryKey,
            key_change_policy: PartitionKeyChangePolicy::EmitMove,
        },
    )
    .expect("partition plan");

    assert_eq!(plan.manifest.global_event_count, 2);
    assert_eq!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks)
            .expect("barrier")
            .iter()
            .map(|change| Operation::try_from(change.operation).expect("operation"))
            .collect::<Vec<_>>(),
        vec![Operation::Delete, Operation::Insert]
    );
}
