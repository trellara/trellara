pub(super) use super::*;

mod chunks;
mod local_view;
mod manifest;
mod operations;

pub(super) fn partition_plan(partition_count: u32) -> PartitionPlan {
    plan_partitioned_transaction(
        &multi_store_envelope(),
        &PartitionPlanConfig {
            partition_count,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan")
}
