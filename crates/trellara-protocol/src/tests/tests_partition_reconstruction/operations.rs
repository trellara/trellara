use super::*;

#[test]
fn barrier_reconstruction_rejects_duplicate_total_order_inside_verified_chunk() {
    let plan = duplicate_total_order_plan();

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::DuplicateTransactionEventOrder { total_order: 3 })
    ));
}

#[test]
fn barrier_reconstruction_rejects_unsupported_change_operation() {
    let mut plan = partition_plan(8);
    plan.chunks[0].changes[0].operation = 99;
    plan.chunks[0].finalize_checksum();
    plan.manifest.partitions[0].checksum = plan.chunks[0].checksum;
    let total_order = plan.chunks[0].changes[0].total_order;

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::UnsupportedOperation {
            total_order: actual,
            operation: 99,
        }) if actual == total_order
    ));
}

#[test]
fn barrier_reconstruction_rejects_unspecified_change_operation() {
    let mut plan = partition_plan(8);
    plan.chunks[0].changes[0].operation = Operation::Unspecified as i32;
    plan.chunks[0].finalize_checksum();
    plan.manifest.partitions[0].checksum = plan.chunks[0].checksum;
    let total_order = plan.chunks[0].changes[0].total_order;

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::UnsupportedOperation {
            total_order: actual,
            operation: 0,
        }) if actual == total_order
    ));
}

fn duplicate_total_order_plan() -> PartitionPlan {
    let mut plan = partition_plan(1);
    plan.chunks[0].changes[1].total_order = plan.chunks[0].changes[2].total_order;
    plan.chunks[0].finalize_checksum();
    plan.manifest.partitions[0].checksum = plan.chunks[0].checksum;
    plan
}
