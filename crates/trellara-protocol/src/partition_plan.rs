use crate::partition_affected_tables::affected_tables_for_changes;
use crate::partition_manifest_plan::{manifest_partition_for_chunk, transaction_manifest};
use crate::partition_plan_chunks::{
    affected_tables_for_chunks, partitioned_chunks, strict_order_chunks,
};
use crate::partition_plan_guards::{ensure_no_ddl_events, ensure_non_empty_transaction};
use crate::{
    ChangeRecord, ManifestBoundaryMode, PartitionPlan, PartitionPlanConfig, ProtocolError,
    StrictChunkPlanConfig, TransactionEnvelope,
};

pub fn plan_partitioned_transaction(
    envelope: &TransactionEnvelope,
    config: &PartitionPlanConfig,
) -> Result<PartitionPlan, ProtocolError> {
    if config.partition_count == 0 {
        return Err(ProtocolError::InvalidPartitionCount);
    }
    ensure_no_ddl_events(envelope, "partitioned scale mode")?;
    ensure_non_empty_transaction(envelope)?;

    let chunks = partitioned_chunks(envelope, config)?;
    let partitions = chunks
        .iter()
        .map(|chunk| manifest_partition_for_chunk(envelope, chunk))
        .collect::<Result<Vec<_>, _>>()?;

    let affected_tables = affected_tables_for_chunks(envelope, &chunks)?;

    Ok(PartitionPlan {
        manifest: transaction_manifest(
            envelope,
            partitions,
            affected_tables,
            &chunks,
            ManifestBoundaryMode::PartitionedScale,
        )?,
        chunks,
    })
}

pub fn plan_strict_chunked_transaction(
    envelope: &TransactionEnvelope,
    config: &StrictChunkPlanConfig,
) -> Result<PartitionPlan, ProtocolError> {
    if config.max_changes_per_chunk == 0 {
        return Err(ProtocolError::InvalidStrictChunkSize);
    }
    ensure_no_ddl_events(envelope, "strict chunked transaction order")?;
    ensure_non_empty_transaction(envelope)?;

    let chunks = strict_order_chunks(envelope, config)?;
    let partitions = chunks
        .iter()
        .map(|chunk| manifest_partition_for_chunk(envelope, chunk))
        .collect::<Result<Vec<_>, _>>()?;
    let affected_tables =
        affected_tables_for_changes(&envelope.transaction_id, envelope.changes.iter())?;

    Ok(PartitionPlan {
        manifest: transaction_manifest(
            envelope,
            partitions,
            affected_tables,
            &chunks,
            ManifestBoundaryMode::StrictChunkedTransactionOrder,
        )?,
        chunks,
    })
}

pub fn partition_local_changes(plan: &PartitionPlan, partition_id: u32) -> Vec<ChangeRecord> {
    plan.chunks
        .iter()
        .find(|chunk| chunk.partition_id == partition_id)
        .map(|chunk| chunk.changes.clone())
        .unwrap_or_default()
}
