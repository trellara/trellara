use super::*;

#[test]
fn partition_planning_rejects_key_change_by_default() {
    let envelope = envelope_with(
        "tx-1",
        vec![ownership_move_change(1, "store-104", "store-205")],
    );

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::PartitionKeyChangeRejected {
            total_order: 1,
            policy: PartitionKeyChangePolicy::Quarantine,
            ..
        })
    ));
}
