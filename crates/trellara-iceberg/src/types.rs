use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::IcebergTableIdentifier;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergFileFormat {
    Parquet,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergCompletedDataFile {
    pub planned_object_key: String,
    pub table_name: String,
    pub relation: String,
    pub source_bucket: u32,
    pub file_uri: String,
    pub file_format: IcebergFileFormat,
    pub file_size_in_bytes: u64,
    pub content_sha256: String,
    pub object_version: Option<String>,
    pub record_count: u64,
    pub checksum_rollup: u64,
}

impl TryFrom<trellara_lake::LakeRawCdcParquetWriteEvidence> for IcebergCompletedDataFile {
    type Error = crate::IcebergIntegrationError;

    fn try_from(
        evidence: trellara_lake::LakeRawCdcParquetWriteEvidence,
    ) -> Result<Self, Self::Error> {
        if evidence.file_format != "parquet" {
            return Err(crate::IcebergIntegrationError::InvalidCompletedDataFile {
                planned_object_key: evidence.planned_object_key,
                field: "file_format",
                reason: "only Parquet lake writer evidence can become Iceberg data-file evidence"
                    .to_string(),
            });
        }
        Ok(Self {
            planned_object_key: evidence.planned_object_key,
            table_name: evidence.table_name,
            relation: evidence.relation,
            source_bucket: evidence.source_bucket,
            file_uri: evidence.file_uri,
            file_format: IcebergFileFormat::Parquet,
            file_size_in_bytes: evidence.file_size_in_bytes,
            content_sha256: evidence.content_sha256,
            object_version: evidence.object_version,
            record_count: evidence.record_count,
            checksum_rollup: evidence.checksum_rollup,
        })
    }
}

impl TryFrom<trellara_lake::LakeRawCdcEpochMetadataParquetWriteEvidence>
    for IcebergCompletedDataFile
{
    type Error = crate::IcebergIntegrationError;

    fn try_from(
        evidence: trellara_lake::LakeRawCdcEpochMetadataParquetWriteEvidence,
    ) -> Result<Self, Self::Error> {
        if evidence.file_format != "parquet" {
            return Err(crate::IcebergIntegrationError::InvalidCompletedDataFile {
                planned_object_key: evidence.planned_object_key,
                field: "file_format",
                reason:
                    "only Parquet epoch metadata evidence can become Iceberg data-file evidence"
                        .to_string(),
            });
        }
        Ok(Self {
            planned_object_key: evidence.planned_object_key,
            table_name: evidence.table_name,
            relation: crate::ICEBERG_EPOCH_METADATA_RELATION.to_string(),
            source_bucket: 0,
            file_uri: evidence.file_uri,
            file_format: IcebergFileFormat::Parquet,
            file_size_in_bytes: evidence.file_size_in_bytes,
            content_sha256: evidence.content_sha256,
            object_version: evidence.object_version,
            record_count: evidence.record_count,
            checksum_rollup: evidence.checksum_rollup,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergCommitRequirements {
    pub require_table_uuid_match: bool,
    pub require_current_snapshot_match: bool,
    pub check_duplicate_files: bool,
}

impl Default for IcebergCommitRequirements {
    fn default() -> Self {
        Self {
            require_table_uuid_match: true,
            require_current_snapshot_match: true,
            check_duplicate_files: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergEpochCommitPlan {
    pub dataset_id: String,
    pub epoch_id: String,
    pub epoch_commit_id: String,
    pub manifest_digest: String,
    pub checksum_rollup: u64,
    pub transaction_count: usize,
    pub change_count: usize,
    pub file_count: usize,
    pub record_count: u64,
    pub accepted_complete_with_gaps: bool,
    pub epoch_visibility_rule: String,
    pub tables: Vec<IcebergTableAppendPlan>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableAppendPlan {
    pub lake_table_name: String,
    pub relation: String,
    pub target: IcebergTableIdentifier,
    pub epoch_commit_id: String,
    pub table_commit_id: String,
    pub commit_uuid: String,
    pub file_count: usize,
    pub record_count: u64,
    pub requirements: IcebergCommitRequirements,
    pub snapshot_properties: BTreeMap<String, String>,
    pub data_files: Vec<IcebergCompletedDataFile>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergTableCommitStatus {
    Committed,
    AlreadyCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableCommitReceipt {
    pub target: IcebergTableIdentifier,
    pub epoch_commit_id: String,
    pub table_commit_id: String,
    pub snapshot_id: i64,
    pub file_count: usize,
    pub record_count: u64,
    pub status: IcebergTableCommitStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergEpochCommitSummary {
    pub dataset_id: String,
    pub epoch_id: String,
    pub epoch_commit_id: String,
    pub ready_for_epoch_metadata: bool,
    pub expected_table_count: usize,
    pub committed_table_count: usize,
    pub newly_committed_table_count: usize,
    pub already_committed_table_count: usize,
    pub missing_tables: Vec<String>,
    pub snapshot_ids: BTreeMap<String, i64>,
    pub epoch_snapshot_reference: Option<String>,
}
