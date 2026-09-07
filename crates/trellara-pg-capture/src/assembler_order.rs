use crate::assembler_buffer::PendingChange;
use crate::assembler_pending_ddl::PendingTransactionDdlEvent;
use crate::{CaptureError, Result};

pub(crate) fn next_total_order(transaction_id: &str, zero_based_index: usize) -> Result<u32> {
    zero_based_index
        .checked_add(1)
        .and_then(|order| u32::try_from(order).ok())
        .ok_or_else(|| CaptureError::TransactionOrderOverflow {
            transaction_id: transaction_id.to_string(),
            max_supported_changes: u32::MAX,
        })
}

pub(crate) fn compact_event_orders(
    transaction_id: &str,
    changes: &[PendingChange],
    ddl_events: &[PendingTransactionDdlEvent],
) -> Result<Vec<u32>> {
    let mut orders = changes
        .iter()
        .map(|change| change.total_order)
        .chain(ddl_events.iter().map(|event| event.total_order))
        .collect::<Vec<_>>();
    orders.sort_unstable();
    if let Some(total_order) = duplicate_order(&orders) {
        return Err(CaptureError::DuplicateTransactionEventOrder {
            transaction_id: transaction_id.to_string(),
            total_order,
        });
    }
    orders.dedup();
    if !orders.is_empty() {
        next_total_order(transaction_id, orders.len() - 1)?;
    }
    Ok(orders)
}

pub(crate) fn compacted_order(order_map: &[u32], original: u32) -> Result<u32> {
    order_map
        .binary_search(&original)
        .ok()
        .and_then(|index| u32::try_from(index + 1).ok())
        .ok_or_else(|| CaptureError::TransactionOrderOverflow {
            transaction_id: "event-order".to_string(),
            max_supported_changes: u32::MAX,
        })
}

fn duplicate_order(sorted_orders: &[u32]) -> Option<u32> {
    sorted_orders
        .windows(2)
        .find(|pair| pair[0] == pair[1])
        .map(|pair| pair[0])
}

#[cfg(test)]
#[path = "tests/tests_assembler_order.rs"]
mod tests;
