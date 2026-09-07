use super::*;
use crate::raw_cdc::metadata::RawCdcMetadataInput;
use std::collections::BTreeSet;

pub(super) fn input() -> RawCdcMetadataInput {
    RawCdcMetadataInput {
        dataset_id: "retail".to_string(),
        epoch_id: "epoch-1".to_string(),
        transaction_count: 1,
        change_count: 2,
        checksum_rollup: 3,
        required_sources: BTreeSet::new(),
        straggler_policy: LakeStragglerPolicy::WaitAllRequired,
        source_rows: Vec::new(),
        table_rows: Vec::new(),
        partition_rows: Vec::new(),
    }
}

pub(super) fn required_sources() -> BTreeSet<String> {
    ["store-001", "store-002"]
        .into_iter()
        .map(ToString::to_string)
        .collect()
}

pub(super) fn source_row(source_id: &str) -> LakeRawCdcEpochSourceRow {
    LakeRawCdcEpochSourceRow {
        epoch_id: "epoch-1".to_string(),
        source_id: source_id.to_string(),
        state: LakeEpochSourceState::Complete,
        start_lsn: "0/16B0000".to_string(),
        end_lsn: "0/16B0100".to_string(),
        transaction_count: 1,
        change_count: 2,
        checksum_rollup: 3,
        lag_reason: None,
    }
}

pub(super) fn source<'a>(
    sources: &'a [LakeRawCdcEpochSourceRow],
    source_id: &str,
) -> &'a LakeRawCdcEpochSourceRow {
    sources
        .iter()
        .find(|source| source.source_id == source_id)
        .expect("source row")
}

pub(super) fn table_row(relation: &str, epoch_id: &str) -> LakeRawCdcEpochTableRow {
    LakeRawCdcEpochTableRow {
        epoch_id: epoch_id.to_string(),
        relation: relation.to_string(),
        transaction_count: 1,
        change_count: 2,
        checksum_rollup: 3,
    }
}

pub(super) fn partition_row(source_id: &str, partition_id: u32) -> LakeRawCdcEpochPartitionRow {
    LakeRawCdcEpochPartitionRow {
        epoch_id: "epoch-1".to_string(),
        source_id: source_id.to_string(),
        partition_id,
        first_commit_lsn: "0/16B0000".to_string(),
        last_commit_lsn: "0/16B0100".to_string(),
        transaction_count: 1,
        event_count: 2,
        checksum_rollup: 3,
    }
}
