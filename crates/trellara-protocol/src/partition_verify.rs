use std::collections::BTreeSet;

use crate::{ChangeRecord, ManifestPartition, PartitionChunk, ProtocolError, TransactionManifest};

pub fn validate_partition_chunk(chunk: &PartitionChunk) -> Result<(), ProtocolError> {
    validate_chunk_transaction_id(chunk)?;
    chunk.verify_checksum()?;
    verify_unique_total_orders(&chunk.changes)?;
    verify_change_transaction_ids(chunk)
}

pub(crate) fn verify_manifest_chunk(
    manifest: &TransactionManifest,
    expected: &ManifestPartition,
    chunk: &PartitionChunk,
) -> Result<(), ProtocolError> {
    validate_chunk_transaction_id(chunk)?;
    if chunk.transaction_id != manifest.transaction_id {
        return Err(ProtocolError::PartitionTransactionMismatch {
            partition_id: expected.id,
            expected: manifest.transaction_id.clone(),
            actual: chunk.transaction_id.clone(),
        });
    }

    let actual_event_count =
        u32::try_from(chunk.changes.len()).map_err(|_| ProtocolError::ManifestCountOverflow {
            transaction_id: manifest.transaction_id.clone(),
            field: "partition.event_count",
            max_supported_count: u32::MAX,
        })?;
    if actual_event_count != expected.event_count {
        return Err(ProtocolError::PartitionEventCountMismatch {
            partition_id: expected.id,
            expected: expected.event_count,
            actual: actual_event_count,
        });
    }
    verify_total_order_range(expected, chunk)?;

    chunk.verify_checksum()?;
    let actual_checksum = chunk.compute_checksum();
    if actual_checksum != expected.checksum {
        return Err(ProtocolError::PartitionChecksumMismatch {
            partition_id: expected.id,
            expected: expected.checksum,
            actual: actual_checksum,
        });
    }
    verify_unique_total_orders(&chunk.changes)?;
    verify_change_transaction_ids(chunk)?;

    Ok(())
}

fn validate_chunk_transaction_id(chunk: &PartitionChunk) -> Result<(), ProtocolError> {
    if chunk.transaction_id.trim().is_empty() {
        return invalid_chunk_field(chunk, "transaction_id", "must not be empty");
    }
    if chunk.transaction_id != chunk.transaction_id.trim() {
        return invalid_chunk_field(
            chunk,
            "transaction_id",
            "must not contain surrounding whitespace",
        );
    }
    Ok(())
}

fn verify_change_transaction_ids(chunk: &PartitionChunk) -> Result<(), ProtocolError> {
    for change in &chunk.changes {
        if change.transaction_id != chunk.transaction_id {
            return Err(ProtocolError::ChangeTransactionMismatch {
                total_order: change.total_order,
                expected: chunk.transaction_id.clone(),
                actual: change.transaction_id.clone(),
            });
        }
    }
    Ok(())
}

fn invalid_chunk_field<T>(
    chunk: &PartitionChunk,
    field: &'static str,
    reason: impl Into<String>,
) -> Result<T, ProtocolError> {
    Err(ProtocolError::InvalidPartitionChunkField {
        partition_id: chunk.partition_id,
        field,
        reason: reason.into(),
    })
}

fn verify_unique_total_orders(changes: &[ChangeRecord]) -> Result<(), ProtocolError> {
    let mut seen = BTreeSet::new();
    for change in changes {
        if !seen.insert(change.total_order) {
            return Err(ProtocolError::DuplicateTransactionEventOrder {
                total_order: change.total_order,
            });
        }
    }
    Ok(())
}

fn verify_total_order_range(
    expected: &ManifestPartition,
    chunk: &PartitionChunk,
) -> Result<(), ProtocolError> {
    let actual_first = chunk
        .changes
        .iter()
        .map(|change| change.total_order)
        .min()
        .unwrap_or_default();
    let actual_last = chunk
        .changes
        .iter()
        .map(|change| change.total_order)
        .max()
        .unwrap_or_default();
    if actual_first == expected.first_total_order && actual_last == expected.last_total_order {
        return Ok(());
    }

    Err(ProtocolError::PartitionTotalOrderRangeMismatch {
        partition_id: expected.id,
        expected_first: expected.first_total_order,
        expected_last: expected.last_total_order,
        actual_first,
        actual_last,
    })
}
