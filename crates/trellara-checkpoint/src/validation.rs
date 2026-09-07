use crate::lsn_validation::validate_required_nonzero_lsn;
use crate::validation_guards::{require_non_empty, require_non_negative, require_positive};
use crate::{
    parse_lsn, CheckpointError, PartitionCheckpoint, ReseedEvent, Result, SnapshotHandoffEvent,
};

pub(crate) use crate::validation_event_validation::validate_validation_event;

pub(crate) fn validate_reseed_event(event: &ReseedEvent) -> Result<()> {
    require_non_empty(&event.source_id, "reseed event source_id must not be empty")?;
    require_non_empty(
        &event.dataset_id,
        "reseed event dataset_id must not be empty",
    )?;
    require_non_empty(
        &event.watermark_lsn,
        "reseed event cannot be recorded without a watermark LSN",
    )?;
    validate_required_nonzero_lsn("reseed event", "watermark_lsn", &event.watermark_lsn)?;
    require_positive(
        event.table_count,
        format!(
            "reseed event must copy at least one table, got {}",
            event.table_count
        ),
    )?;
    require_non_negative(
        event.copied_rows,
        format!(
            "reseed event cannot record negative copied rows {}",
            event.copied_rows
        ),
    )?;

    Ok(())
}

pub(crate) fn validate_snapshot_handoff_event(event: &SnapshotHandoffEvent) -> Result<()> {
    require_non_empty(
        &event.source_id,
        "snapshot handoff event source_id must not be empty",
    )?;
    require_non_empty(
        &event.dataset_id,
        "snapshot handoff event dataset_id must not be empty",
    )?;
    require_non_empty(
        &event.relation,
        "snapshot handoff event relation must not be empty",
    )?;
    require_non_empty(
        &event.watermark_lsn,
        format!(
            "snapshot handoff event for {} cannot be recorded without a watermark LSN",
            event.relation
        ),
    )?;
    validate_required_nonzero_lsn(
        "snapshot handoff event",
        "watermark_lsn",
        &event.watermark_lsn,
    )?;
    require_non_negative(
        event.copied_rows,
        format!(
            "snapshot handoff event for {} cannot record negative copied rows {}",
            event.relation, event.copied_rows
        ),
    )?;

    Ok(())
}

pub(crate) fn validate_partition_checkpoint(checkpoint: &PartitionCheckpoint) -> Result<()> {
    require_non_empty(
        &checkpoint.source_id,
        "partition checkpoint source_id must not be empty",
    )?;
    require_non_empty(
        &checkpoint.dataset_id,
        "partition checkpoint dataset_id must not be empty",
    )?;
    if checkpoint.last_durable_lsn.trim().is_empty()
        && checkpoint.last_applied_lsn.trim().is_empty()
    {
        return Err(CheckpointError::Store(format!(
            "partition checkpoint {} cannot be recorded without a durable or applied LSN",
            checkpoint.partition_id
        )));
    }
    validate_partition_checkpoint_lsn(
        checkpoint.partition_id,
        "last_durable_lsn",
        &checkpoint.last_durable_lsn,
    )?;
    validate_partition_checkpoint_lsn(
        checkpoint.partition_id,
        "last_applied_lsn",
        &checkpoint.last_applied_lsn,
    )?;
    if !checkpoint.last_durable_lsn.trim().is_empty()
        && !checkpoint.last_applied_lsn.trim().is_empty()
        && parse_lsn(&checkpoint.last_applied_lsn) > parse_lsn(&checkpoint.last_durable_lsn)
    {
        return Err(CheckpointError::Store(format!(
            "partition checkpoint {} applied LSN {} is ahead of durable LSN {}",
            checkpoint.partition_id, checkpoint.last_applied_lsn, checkpoint.last_durable_lsn
        )));
    }

    Ok(())
}

fn validate_partition_checkpoint_lsn(partition_id: u32, field: &str, lsn: &str) -> Result<()> {
    if lsn.trim().is_empty() {
        return Ok(());
    }
    validate_required_nonzero_lsn(&format!("partition checkpoint {partition_id}"), field, lsn)
}
