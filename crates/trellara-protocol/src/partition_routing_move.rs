use super::partition_routing_target::partition_for_row;
use super::RoutedPartitionChange;
use crate::{ChangeRecord, Operation, PartitionPlanConfig, ProtocolError};

pub(crate) fn emit_move_changes(
    change: &ChangeRecord,
    config: &PartitionPlanConfig,
) -> Result<Vec<RoutedPartitionChange>, ProtocolError> {
    let before = change
        .before
        .as_ref()
        .ok_or(ProtocolError::MissingPartitionRowImage {
            total_order: change.total_order,
        })?;
    let after = change
        .after
        .as_ref()
        .ok_or(ProtocolError::MissingPartitionRowImage {
            total_order: change.total_order,
        })?;

    Ok(vec![
        RoutedPartitionChange {
            partition: partition_for_row(before, change.total_order, config)?,
            change: move_delete_change(change),
        },
        RoutedPartitionChange {
            partition: partition_for_row(after, change.total_order, config)?,
            change: move_insert_change(change),
        },
    ])
}

fn move_delete_change(change: &ChangeRecord) -> ChangeRecord {
    let mut delete = change.clone();
    delete.operation = Operation::Delete as i32;
    delete.after = None;
    delete.idempotency_key = format!("{}:move_delete", change.idempotency_key);
    delete
}

fn move_insert_change(change: &ChangeRecord) -> ChangeRecord {
    let mut insert = change.clone();
    insert.operation = Operation::Insert as i32;
    insert.before = None;
    insert.idempotency_key = format!("{}:move_insert", change.idempotency_key);
    insert
}
