pub(super) use super::*;

mod construction;
mod manifest_validation;
mod matching;
mod reconstruction;

pub(super) fn partitioned_multi_store_plan() -> (TransactionEnvelope, PartitionPlan) {
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

    (envelope, plan)
}
