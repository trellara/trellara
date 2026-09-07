use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    IcebergEpochCommitPlan, IcebergRawCdcColumn, IcebergRawCdcPartitionField,
    IcebergRawCdcTableSpec, IcebergTableIdentifier,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergEpochMetadataTableSpec {
    pub lake_table_name: String,
    pub relation: String,
    pub target: IcebergTableIdentifier,
    pub schema_fingerprint_sha256: String,
    pub columns: Vec<IcebergRawCdcColumn>,
    pub partition_fields: Vec<IcebergRawCdcPartitionField>,
    pub write_mode: String,
    pub visibility_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergEpochMetadataAppendPlan {
    pub dataset_id: String,
    pub epoch_id: String,
    pub epoch_commit_id: String,
    pub epoch_snapshot_reference: String,
    pub raw_snapshot_ids: BTreeMap<String, i64>,
    pub metadata_table: String,
    pub metadata_commit_plan: IcebergEpochCommitPlan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergFaninL1TableSpecs {
    pub dataset_id: String,
    pub epoch_id: String,
    pub completeness_contract: String,
    pub raw_changelog_tables: Vec<IcebergRawCdcTableSpec>,
    pub epoch_metadata_table: IcebergEpochMetadataTableSpec,
}
