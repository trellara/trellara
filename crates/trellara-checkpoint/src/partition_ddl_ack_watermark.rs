use std::collections::BTreeSet;

use crate::{
    lsn::lsn_shape_is_valid, parse_lsn, CheckpointError, PartitionWatermarkLag,
    PartitionWatermarkSummary, Result,
};

pub(crate) fn validate_watermark_summary_consistency(
    watermarks: &PartitionWatermarkSummary,
) -> Result<()> {
    let observed = u32::try_from(watermarks.partitions.len()).map_err(|_| {
        store_error(format!(
            "partition visibility DDL ack observed partition evidence count {} exceeds supported u32 range",
            watermarks.partitions.len()
        ))
    })?;
    if watermarks.observed_partition_count != observed {
        return Err(store_error(format!(
            "partition visibility DDL ack observed_partition_count {} does not match {} partition evidence rows",
            watermarks.observed_partition_count, observed
        )));
    }
    if watermarks.complete_partition_set
        && watermarks.observed_partition_count != watermarks.expected_partition_count
    {
        return Err(store_error(format!(
            "partition visibility DDL ack complete_partition_set requires observed_partition_count {} to match expected_partition_count {}",
            watermarks.observed_partition_count, watermarks.expected_partition_count
        )));
    }
    if watermarks.complete_partition_set && !watermarks.missing_partitions.is_empty() {
        return Err(store_error(format!(
            "partition visibility DDL ack complete_partition_set cannot include missing partitions {:?}",
            watermarks.missing_partitions
        )));
    }
    if watermarks.complete_partition_set && watermarks.global_durable_lsn.is_none() {
        return Err(store_error(
            "partition visibility DDL ack is missing global_durable_lsn",
        ));
    }
    if watermarks.complete_partition_set && watermarks.global_applied_lsn.is_none() {
        return Err(store_error(
            "partition visibility DDL ack is missing global_applied_lsn",
        ));
    }
    validate_partition_rows(watermarks)?;
    validate_global_minimums(watermarks)?;
    Ok(())
}

fn validate_partition_rows(watermarks: &PartitionWatermarkSummary) -> Result<()> {
    let mut seen = BTreeSet::new();
    for partition in &watermarks.partitions {
        if !seen.insert(partition.partition_id) {
            return Err(store_error(format!(
                "partition visibility DDL ack includes duplicate partition {}",
                partition.partition_id
            )));
        }
        if partition.partition_id >= watermarks.expected_partition_count {
            return Err(store_error(format!(
                "partition visibility DDL ack partition {} is outside expected range 0..{}",
                partition.partition_id,
                watermarks.expected_partition_count.saturating_sub(1)
            )));
        }
        validate_partition_lsn(partition, "last_durable_lsn", &partition.last_durable_lsn)?;
        validate_partition_lsn(partition, "last_applied_lsn", &partition.last_applied_lsn)?;
        if parse_lsn(&partition.last_applied_lsn) > parse_lsn(&partition.last_durable_lsn) {
            return Err(store_error(format!(
                "partition visibility DDL ack partition {} applied LSN {} is ahead of durable LSN {}",
                partition.partition_id, partition.last_applied_lsn, partition.last_durable_lsn
            )));
        }
    }
    Ok(())
}

fn validate_global_minimums(watermarks: &PartitionWatermarkSummary) -> Result<()> {
    if !watermarks.complete_partition_set {
        return Ok(());
    }
    let Some(durable_min) = lowest_lsn(watermarks.partitions.iter().map(|p| &p.last_durable_lsn))
    else {
        return Ok(());
    };
    let Some(applied_min) = lowest_lsn(watermarks.partitions.iter().map(|p| &p.last_applied_lsn))
    else {
        return Ok(());
    };
    if watermarks.global_durable_lsn.as_deref() != Some(durable_min) {
        return Err(store_error("partition visibility DDL ack global_durable_lsn must match the lowest per-partition durable LSN"));
    }
    if watermarks.global_applied_lsn.as_deref() != Some(applied_min) {
        return Err(store_error("partition visibility DDL ack global_applied_lsn must match the lowest per-partition applied LSN"));
    }
    Ok(())
}

fn validate_partition_lsn(
    partition: &PartitionWatermarkLag,
    field: &'static str,
    lsn: &str,
) -> Result<()> {
    if lsn.trim() != lsn {
        return Err(store_error(format!(
            "partition visibility DDL ack partition {} {field} must not contain surrounding whitespace",
            partition.partition_id
        )));
    }
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(store_error(format!(
            "partition visibility DDL ack partition {} {field} {lsn} must be a non-zero PostgreSQL LSN like 0/16B9000",
            partition.partition_id
        )));
    }
    Ok(())
}

fn lowest_lsn<'a>(lsns: impl Iterator<Item = &'a String>) -> Option<&'a str> {
    lsns.min_by_key(|lsn| parse_lsn(lsn)).map(String::as_str)
}

fn store_error(message: impl Into<String>) -> CheckpointError {
    CheckpointError::Store(message.into())
}
