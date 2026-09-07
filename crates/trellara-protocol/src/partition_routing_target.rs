use super::partition_for_key;
use crate::partition_keys::{
    partition_key_bytes, partition_key_bytes_from_row, primary_key_bytes_from_row,
};
use crate::{ChangeRecord, PartitionNullKeyPolicy, PartitionPlanConfig, ProtocolError, RowImage};

pub(super) fn partition_for_change(
    change: &ChangeRecord,
    config: &PartitionPlanConfig,
) -> Result<u32, ProtocolError> {
    match partition_key_bytes(change, &config.key_column) {
        Ok(key) => Ok(partition_for_key(&key, config.partition_count)),
        Err(ProtocolError::NullPartitionKey {
            total_order,
            column,
        }) => match config.null_key_policy {
            PartitionNullKeyPolicy::RouteToSingletonPartition => Ok(0),
            PartitionNullKeyPolicy::RouteToDeadLetterPartition => Ok(config.partition_count - 1),
            PartitionNullKeyPolicy::DeriveFromPrimaryKey => {
                let row = change.after.as_ref().or(change.before.as_ref()).ok_or(
                    ProtocolError::MissingPartitionRowImage {
                        total_order: change.total_order,
                    },
                )?;
                Ok(partition_for_key(
                    &primary_key_bytes_from_row(row, total_order)?,
                    config.partition_count,
                ))
            }
            PartitionNullKeyPolicy::Quarantine => Err(ProtocolError::NullPartitionKey {
                total_order,
                column,
            }),
        },
        Err(error) => Err(error),
    }
}

pub(super) fn partition_for_row(
    row: &RowImage,
    total_order: u32,
    config: &PartitionPlanConfig,
) -> Result<u32, ProtocolError> {
    match partition_key_bytes_from_row(row, total_order, &config.key_column) {
        Ok(key) => Ok(partition_for_key(&key, config.partition_count)),
        Err(ProtocolError::NullPartitionKey { column, .. }) => match config.null_key_policy {
            PartitionNullKeyPolicy::RouteToSingletonPartition => Ok(0),
            PartitionNullKeyPolicy::RouteToDeadLetterPartition => Ok(config.partition_count - 1),
            PartitionNullKeyPolicy::DeriveFromPrimaryKey => Ok(partition_for_key(
                &primary_key_bytes_from_row(row, total_order)?,
                config.partition_count,
            )),
            PartitionNullKeyPolicy::Quarantine => Err(ProtocolError::NullPartitionKey {
                total_order,
                column,
            }),
        },
        Err(error) => Err(error),
    }
}
