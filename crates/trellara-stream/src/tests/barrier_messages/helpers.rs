use super::*;

pub(super) fn partition_plan() -> PartitionPlan {
    plan_partitioned_transaction(
        &sample_envelope(),
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan")
}

pub(super) fn commit_marker(plan: &PartitionPlan) -> TransactionCommitMarker {
    TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker")
}

pub(super) fn assert_barrier_payload_mismatch(
    result: Result<StreamMessage>,
    expected_field: &'static str,
) {
    assert!(matches!(
        result,
        Err(StreamError::BarrierPayloadMismatch { field, .. }) if field == expected_field
    ));
}
