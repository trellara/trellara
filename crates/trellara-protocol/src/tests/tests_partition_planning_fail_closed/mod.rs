pub(super) use super::*;

mod ddl_boundaries;
mod key_changes;
mod key_values;
mod operations;

pub(super) fn partition_config(null_key_policy: PartitionNullKeyPolicy) -> PartitionPlanConfig {
    PartitionPlanConfig {
        partition_count: 8,
        key_column: "store_id".to_string(),
        null_key_policy,
        key_change_policy: PartitionKeyChangePolicy::Quarantine,
    }
}

pub(super) fn envelope_with(
    transaction_id: &str,
    changes: Vec<ChangeRecord>,
) -> TransactionEnvelope {
    TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: transaction_id.to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes,
    })
}

pub(super) fn default_partition_config() -> PartitionPlanConfig {
    partition_config(PartitionNullKeyPolicy::Quarantine)
}

pub(super) fn ddl_event(transaction_id: &str, total_order: u32) -> DdlEvent {
    DdlEvent::additive_column(
        transaction_id,
        total_order,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )
}
