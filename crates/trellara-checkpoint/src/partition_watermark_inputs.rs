use std::collections::HashSet;

use crate::{lsn::lsn_shape_is_valid, parse_lsn, CheckpointError, PartitionCheckpoint, Result};

pub(crate) struct ValidatedPartitionWatermarkInputs {
    pub(crate) checkpoints: Vec<PartitionCheckpoint>,
    pub(crate) missing_partitions: Vec<u32>,
}

pub(crate) fn validate_partition_watermark_inputs(
    source_id: &str,
    dataset_id: &str,
    expected_partition_count: u32,
    mut checkpoints: Vec<PartitionCheckpoint>,
) -> Result<ValidatedPartitionWatermarkInputs> {
    if let Some(mismatched) = checkpoints
        .iter()
        .find(|checkpoint| checkpoint.source_id != source_id || checkpoint.dataset_id != dataset_id)
    {
        return Err(CheckpointError::Store(format!(
            "partition checkpoint {} belongs to {}.{}, expected {}.{}",
            mismatched.partition_id,
            mismatched.source_id,
            mismatched.dataset_id,
            source_id,
            dataset_id
        )));
    }

    if let Some(missing_lsn) = checkpoints.iter().find(|checkpoint| {
        checkpoint.last_durable_lsn.trim().is_empty()
            || checkpoint.last_applied_lsn.trim().is_empty()
    }) {
        return Err(CheckpointError::Store(format!(
            "partition checkpoint {} is missing durable or applied LSN evidence",
            missing_lsn.partition_id
        )));
    }

    for checkpoint in &checkpoints {
        let durable = validate_lsn(
            checkpoint.partition_id,
            "last_durable_lsn",
            &checkpoint.last_durable_lsn,
        )?;
        let applied = validate_lsn(
            checkpoint.partition_id,
            "last_applied_lsn",
            &checkpoint.last_applied_lsn,
        )?;
        if applied > durable {
            return Err(CheckpointError::Store(format!(
                "partition checkpoint {} applied LSN {} is ahead of durable LSN {}",
                checkpoint.partition_id, checkpoint.last_applied_lsn, checkpoint.last_durable_lsn
            )));
        }
    }

    checkpoints.sort_by_key(|checkpoint| checkpoint.partition_id);
    for window in checkpoints.windows(2) {
        if window[0].partition_id == window[1].partition_id {
            return Err(CheckpointError::Store(format!(
                "duplicate checkpoint for partition {}",
                window[0].partition_id
            )));
        }
    }

    if let Some(out_of_range) = checkpoints
        .iter()
        .find(|checkpoint| checkpoint.partition_id >= expected_partition_count)
    {
        return Err(CheckpointError::Store(format!(
            "partition {} is outside expected range 0..{}",
            out_of_range.partition_id,
            expected_partition_count.saturating_sub(1)
        )));
    }

    let observed = checkpoints
        .iter()
        .map(|checkpoint| checkpoint.partition_id)
        .collect::<HashSet<_>>();
    let missing_partitions = (0..expected_partition_count)
        .filter(|partition_id| !observed.contains(partition_id))
        .collect::<Vec<_>>();

    Ok(ValidatedPartitionWatermarkInputs {
        checkpoints,
        missing_partitions,
    })
}

fn validate_lsn(partition_id: u32, field: &'static str, lsn: &str) -> Result<u64> {
    if lsn.trim() != lsn {
        return Err(CheckpointError::Store(format!(
            "partition checkpoint {partition_id} {field} must not contain surrounding whitespace"
        )));
    }
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(CheckpointError::Store(format!(
            "partition checkpoint {partition_id} {field} {lsn} must be a non-zero PostgreSQL LSN like 0/16B9000"
        )));
    }
    Ok(parse_lsn(lsn))
}
