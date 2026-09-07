#[path = "lake_epoch_digest.rs"]
mod lake_epoch_digest;
#[path = "lake_epoch_watermark_rollup.rs"]
mod lake_epoch_watermark_rollup;

use trellara_sim::{
    FleetFanInPartitionRollup, FleetFanInQuarantineEntry, FleetFanInSourceWatermark,
    FleetFanInTableRollup,
};

use crate::{
    LakeEpochPartitionRollup, LakeEpochQuarantineEntry, LakeEpochSourceWatermark,
    LakeEpochTableRollup,
};

pub(crate) use lake_epoch_digest::*;
pub(crate) use lake_epoch_watermark_rollup::*;

pub(crate) fn lake_epoch_source_watermarks(
    sources: &[FleetFanInSourceWatermark],
) -> Vec<LakeEpochSourceWatermark> {
    sources
        .iter()
        .map(|source| LakeEpochSourceWatermark {
            source_id: source.source_id.clone(),
            state: source.state.clone(),
            start_lsn: source.start_lsn.clone(),
            end_lsn: source.end_lsn.clone(),
            transaction_count: source.transaction_count,
            change_count: source.change_count,
            gap_reason: source.gap_reason.clone(),
        })
        .collect()
}

pub(crate) fn lake_epoch_table_rollups(
    tables: &[FleetFanInTableRollup],
) -> Vec<LakeEpochTableRollup> {
    tables
        .iter()
        .map(|table| LakeEpochTableRollup {
            relation: table.relation.clone(),
            transaction_count: table.transaction_count,
            change_count: table.change_count,
            checksum_rollup: table.checksum_rollup,
        })
        .collect()
}

pub(crate) fn lake_epoch_partition_rollups(
    partitions: &[FleetFanInPartitionRollup],
) -> Vec<LakeEpochPartitionRollup> {
    partitions
        .iter()
        .map(|partition| LakeEpochPartitionRollup {
            source_id: partition.source_id.clone(),
            partition_id: partition.partition_id,
            first_commit_lsn: partition.first_commit_lsn.clone(),
            last_commit_lsn: partition.last_commit_lsn.clone(),
            transaction_count: partition.transaction_count,
            event_count: partition.event_count,
            checksum_rollup: partition.checksum_rollup,
        })
        .collect()
}

pub(crate) fn lake_epoch_quarantine_entries(
    entries: &[FleetFanInQuarantineEntry],
) -> Vec<LakeEpochQuarantineEntry> {
    entries
        .iter()
        .map(|entry| LakeEpochQuarantineEntry {
            source_id: entry.source_id.clone(),
            transaction_id: entry.transaction_id.clone(),
            commit_lsn: entry.commit_lsn.clone(),
            reason: entry.reason.clone(),
            details: entry.details.clone(),
            recovery_command: entry.recovery_command.clone(),
        })
        .collect()
}
