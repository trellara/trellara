use super::*;
use trellara_protocol::{
    partition_for_key, plan_partitioned_transaction, ChangeRecord, ColumnValue, Operation,
    PartitionKeyChangePolicy, PartitionNullKeyPolicy, PartitionPlanConfig, RelationId,
    ReplicaIdentity, RowImage, StrictEnvelope, TransactionEnvelope,
};

pub(super) fn partitioned_plan(envelope: &TransactionEnvelope) -> trellara_protocol::PartitionPlan {
    let plan = plan_partitioned_transaction(
        envelope,
        &PartitionPlanConfig {
            partition_count: 4,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    assert!(plan.chunks.len() > 1, "fixture should span partitions");
    plan
}

pub(super) fn replace_header(message: &mut StreamMessage, key: &str, value: &str) {
    let header = message
        .headers
        .iter_mut()
        .find(|header| header.key == key)
        .expect("header");
    header.value = value.to_string();
}

pub(super) fn remove_header(message: &mut StreamMessage, key: &str) {
    message.headers.retain(|header| header.key != key);
}

pub(super) fn partitioned_envelope() -> TransactionEnvelope {
    let values = partition_spanning_values();
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source_a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-partitioned".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: values
            .into_iter()
            .enumerate()
            .map(|(index, value)| change(index as u32 + 1, &value))
            .collect(),
    })
}

fn partition_spanning_values() -> Vec<String> {
    let mut values = Vec::new();
    let mut first_partition = None;
    for index in 0..100 {
        let value = format!("store-{index}");
        let partition = partition_for_key(value.as_bytes(), 4);
        first_partition.get_or_insert(partition);
        values.push(value);
        if Some(partition) != first_partition && values.len() >= 4 {
            return values;
        }
    }
    panic!("fixture could not find keys spanning multiple partitions");
}

fn change(total_order: u32, store_id: &str) -> ChangeRecord {
    ChangeRecord {
        transaction_id: "tx-partitioned".to_string(),
        total_order,
        table_order: total_order,
        partition_order: total_order,
        relation: Some(RelationId::new(42, "public", "sales")),
        operation: Operation::Insert as i32,
        replica_identity: ReplicaIdentity::Default as i32,
        before: None,
        after: Some(RowImage::new(vec![ColumnValue::text(
            "store_id", 25, store_id, true,
        )])),
        idempotency_key: format!("source_a:0/16B6C50:tx-partitioned:{total_order}"),
    }
}
