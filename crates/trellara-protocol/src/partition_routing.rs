use xxhash_rust::xxh3::xxh3_64;

#[path = "partition_routing_change.rs"]
mod partition_routing_change;
#[path = "partition_routing_move.rs"]
mod partition_routing_move;
#[path = "partition_routing_target.rs"]
mod partition_routing_target;

use partition_routing_change::{change_operation, partition_key_changed};
use partition_routing_move::emit_move_changes;
use partition_routing_target::partition_for_change;

use crate::{ChangeRecord, PartitionKeyChangePolicy, PartitionPlanConfig, ProtocolError};

pub fn partition_for_key(key: &[u8], partition_count: u32) -> u32 {
    assert!(
        partition_count > 0,
        "partition_count must be greater than zero"
    );
    (xxh3_64(key) % u64::from(partition_count)) as u32
}

pub(crate) struct RoutedPartitionChange {
    pub(crate) partition: u32,
    pub(crate) change: ChangeRecord,
}

pub(crate) fn routed_partition_changes(
    change: &ChangeRecord,
    config: &PartitionPlanConfig,
) -> Result<Vec<RoutedPartitionChange>, ProtocolError> {
    let operation = change_operation(change)?;
    if partition_key_changed(change, operation, &config.key_column)? {
        return match config.key_change_policy {
            PartitionKeyChangePolicy::EmitMove => emit_move_changes(change, config),
            PartitionKeyChangePolicy::Quarantine
            | PartitionKeyChangePolicy::DualWriteWindow
            | PartitionKeyChangePolicy::Forbid => Err(ProtocolError::PartitionKeyChangeRejected {
                total_order: change.total_order,
                column: config.key_column.clone(),
                policy: config.key_change_policy,
            }),
        };
    }

    Ok(vec![RoutedPartitionChange {
        partition: partition_for_change(change, config)?,
        change: change.clone(),
    }])
}
