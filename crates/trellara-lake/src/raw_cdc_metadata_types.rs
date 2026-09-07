use serde::{Deserialize, Serialize};

use crate::{LakeCompletenessState, LakeEpochSourceState, LakeEpochVerificationStatus};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochMetadataPlan {
    pub epochs_table: String,
    pub epoch_sources_table: String,
    pub epoch_tables_table: String,
    pub epoch_partitions_table: String,
    pub quarantine_table: String,
    pub verification_table: String,
    pub epoch_row: LakeRawCdcEpochRow,
    pub source_rows: Vec<LakeRawCdcEpochSourceRow>,
    pub table_rows: Vec<LakeRawCdcEpochTableRow>,
    pub partition_rows: Vec<LakeRawCdcEpochPartitionRow>,
    pub quarantine_rows: Vec<LakeRawCdcEpochQuarantineRow>,
    pub verification_row: LakeRawCdcEpochVerificationRow,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochRow {
    pub epoch_id: String,
    pub dataset_id: String,
    pub state: LakeCompletenessState,
    pub policy: String,
    pub opened_at: String,
    pub sealed_at: String,
    pub required_source_count: usize,
    pub complete_source_count: usize,
    pub missing_source_count: usize,
    pub quarantined_source_count: usize,
    pub transaction_count: usize,
    pub change_count: usize,
    pub checksum_rollup: u64,
    pub manifest_digest: String,
    pub iceberg_snapshot_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochSourceRow {
    pub epoch_id: String,
    pub source_id: String,
    pub state: LakeEpochSourceState,
    pub start_lsn: String,
    pub end_lsn: String,
    pub transaction_count: usize,
    pub change_count: usize,
    pub checksum_rollup: u64,
    pub lag_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochTableRow {
    pub epoch_id: String,
    pub relation: String,
    pub transaction_count: usize,
    pub change_count: usize,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochPartitionRow {
    pub epoch_id: String,
    pub source_id: String,
    pub partition_id: u32,
    pub first_commit_lsn: String,
    pub last_commit_lsn: String,
    pub transaction_count: usize,
    pub event_count: usize,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochQuarantineRow {
    pub epoch_id: String,
    pub source_id: Option<String>,
    pub transaction_id: Option<String>,
    pub commit_lsn: Option<String>,
    pub reason: String,
    pub details: Option<String>,
    pub recovery_command: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochVerificationRow {
    pub epoch_id: String,
    pub verification_id: String,
    pub input_transaction_count: usize,
    pub input_change_count: usize,
    pub lake_transaction_count: usize,
    pub lake_change_count: usize,
    pub checksum_status: LakeEpochVerificationStatus,
    pub completed_at: String,
}
