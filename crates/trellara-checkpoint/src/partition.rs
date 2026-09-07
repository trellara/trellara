use serde::{Deserialize, Serialize};

use crate::lsn::parse_lsn;
use crate::partition_watermark_inputs::validate_partition_watermark_inputs;
use crate::{CheckpointError, Result};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionCheckpoint {
    pub source_id: String,
    pub dataset_id: String,
    pub partition_id: u32,
    pub last_durable_lsn: String,
    pub last_applied_lsn: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionWatermarkSummary {
    pub source_id: String,
    pub dataset_id: String,
    pub expected_partition_count: u32,
    pub observed_partition_count: u32,
    pub complete_partition_set: bool,
    pub global_durable_lsn: Option<String>,
    pub global_applied_lsn: Option<String>,
    pub global_durable_to_applied_bytes: Option<u64>,
    pub missing_partitions: Vec<u32>,
    pub partitions: Vec<PartitionWatermarkLag>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionWatermarkLag {
    pub partition_id: u32,
    pub last_durable_lsn: String,
    pub last_applied_lsn: String,
    pub durable_to_applied_bytes: u64,
    pub blocks_global_applied_watermark: bool,
}

impl PartitionWatermarkSummary {
    pub fn from_checkpoints(
        source_id: impl Into<String>,
        dataset_id: impl Into<String>,
        expected_partition_count: u32,
        checkpoints: Vec<PartitionCheckpoint>,
    ) -> Result<Self> {
        if expected_partition_count == 0 {
            return Err(CheckpointError::Store(
                "expected partition count must be greater than zero".to_string(),
            ));
        }

        let source_id = source_id.into();
        let dataset_id = dataset_id.into();
        let inputs = validate_partition_watermark_inputs(
            &source_id,
            &dataset_id,
            expected_partition_count,
            checkpoints,
        )?;
        let checkpoints = inputs.checkpoints;
        let missing_partitions = inputs.missing_partitions;
        let complete_partition_set = missing_partitions.is_empty();

        let global_durable_lsn = complete_partition_set
            .then(|| {
                lowest_lsn(
                    checkpoints
                        .iter()
                        .map(|checkpoint| &checkpoint.last_durable_lsn),
                )
            })
            .flatten();
        let global_applied_lsn = complete_partition_set
            .then(|| {
                lowest_lsn(
                    checkpoints
                        .iter()
                        .map(|checkpoint| &checkpoint.last_applied_lsn),
                )
            })
            .flatten();
        let global_durable_to_applied_bytes = global_durable_lsn
            .as_ref()
            .zip(global_applied_lsn.as_ref())
            .map(|(durable, applied)| parse_lsn(durable).saturating_sub(parse_lsn(applied)));
        let global_applied_value = global_applied_lsn.as_deref().map(parse_lsn);
        let max_applied_value = checkpoints
            .iter()
            .map(|checkpoint| parse_lsn(&checkpoint.last_applied_lsn))
            .max();

        let partitions = checkpoints
            .into_iter()
            .map(|checkpoint| {
                let applied = parse_lsn(&checkpoint.last_applied_lsn);
                PartitionWatermarkLag {
                    partition_id: checkpoint.partition_id,
                    durable_to_applied_bytes: parse_lsn(&checkpoint.last_durable_lsn)
                        .saturating_sub(applied),
                    blocks_global_applied_watermark: global_applied_value
                        .zip(max_applied_value)
                        .map(|(global, max)| applied == global && applied < max)
                        .unwrap_or(false),
                    last_durable_lsn: checkpoint.last_durable_lsn,
                    last_applied_lsn: checkpoint.last_applied_lsn,
                }
            })
            .collect::<Vec<_>>();

        Ok(Self {
            source_id,
            dataset_id,
            expected_partition_count,
            observed_partition_count: observed_partition_count(partitions.len())?,
            complete_partition_set,
            global_durable_lsn,
            global_applied_lsn,
            global_durable_to_applied_bytes,
            missing_partitions,
            partitions,
        })
    }
}

pub(crate) fn observed_partition_count(count: usize) -> Result<u32> {
    u32::try_from(count).map_err(|_| {
        CheckpointError::Store(format!(
            "observed partition count {count} exceeds supported u32 range"
        ))
    })
}

fn lowest_lsn<'a>(lsns: impl Iterator<Item = &'a String>) -> Option<String> {
    lsns.min_by_key(|lsn| parse_lsn(lsn)).cloned()
}
