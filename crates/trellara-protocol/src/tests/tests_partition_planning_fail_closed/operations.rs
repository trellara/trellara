use super::*;

#[test]
fn partition_planning_rejects_unsupported_change_operation() {
    let mut change = sale_change(1, "store-104");
    change.operation = 99;
    let envelope = envelope_with("tx-1", vec![change]);

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::UnsupportedOperation {
            total_order: 1,
            operation: 99,
        })
    ));
}

#[test]
fn partition_planning_rejects_unspecified_change_operation() {
    let mut change = sale_change(1, "store-104");
    change.operation = Operation::Unspecified as i32;
    let envelope = envelope_with("tx-1", vec![change]);

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::UnsupportedOperation {
            total_order: 1,
            operation: 0,
        })
    ));
}
