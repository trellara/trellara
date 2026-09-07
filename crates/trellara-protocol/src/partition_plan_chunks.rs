use std::collections::BTreeMap;

use crate::partition_affected_tables::affected_tables_for_changes;
use crate::partition_routing::routed_partition_changes;
use crate::{
    AffectedTable, ChangeRecord, PartitionChunk, PartitionPlanConfig, ProtocolError,
    StrictChunkPlanConfig, TransactionEnvelope,
};

pub(crate) fn partitioned_chunks(
    envelope: &TransactionEnvelope,
    config: &PartitionPlanConfig,
) -> Result<Vec<PartitionChunk>, ProtocolError> {
    let mut chunks_by_partition: BTreeMap<u32, Vec<ChangeRecord>> = BTreeMap::new();

    for change in &envelope.changes {
        for routed in routed_partition_changes(change, config)? {
            chunks_by_partition
                .entry(routed.partition)
                .or_default()
                .push(routed.change);
        }
    }

    Ok(chunks_by_partition
        .into_iter()
        .map(|(partition_id, changes)| {
            PartitionChunk::new(envelope.transaction_id.clone(), partition_id, changes)
        })
        .collect())
}

pub(crate) fn strict_order_chunks(
    envelope: &TransactionEnvelope,
    config: &StrictChunkPlanConfig,
) -> Result<Vec<PartitionChunk>, ProtocolError> {
    envelope
        .changes
        .chunks(config.max_changes_per_chunk as usize)
        .enumerate()
        .map(|(chunk_id, changes)| {
            let chunk_id =
                u32::try_from(chunk_id).map_err(|_| ProtocolError::ManifestCountOverflow {
                    transaction_id: envelope.transaction_id.clone(),
                    field: "chunk_id",
                    max_supported_count: u32::MAX,
                })?;
            Ok(PartitionChunk::new(
                envelope.transaction_id.clone(),
                chunk_id,
                changes.to_vec(),
            ))
        })
        .collect()
}

pub(crate) fn affected_tables_for_chunks(
    envelope: &TransactionEnvelope,
    chunks: &[PartitionChunk],
) -> Result<Vec<AffectedTable>, ProtocolError> {
    affected_tables_for_changes(
        &envelope.transaction_id,
        chunks.iter().flat_map(|chunk| chunk.changes.iter()),
    )
}
