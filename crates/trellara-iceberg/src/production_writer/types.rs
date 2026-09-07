use serde::{Deserialize, Serialize};

use crate::{
    IcebergEpochCommitPlan, IcebergMetadataCommitBundle, IcebergTableCommitReceipt,
    IcebergTableProvisioningOutcome, IcebergUploadedMetadataFile, IcebergUploadedRawDataFile,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProductionIcebergEpochResult {
    pub raw_table_provisioning: Vec<IcebergTableProvisioningOutcome>,
    pub metadata_table_provisioning: Vec<IcebergTableProvisioningOutcome>,
    pub raw_uploads: Vec<IcebergUploadedRawDataFile>,
    pub raw_commit_plan: IcebergEpochCommitPlan,
    pub raw_receipts: Vec<IcebergTableCommitReceipt>,
    pub metadata_uploads: Vec<IcebergUploadedMetadataFile>,
    pub metadata_commit_bundle: IcebergMetadataCommitBundle,
    pub supporting_metadata_receipts: Vec<IcebergTableCommitReceipt>,
    pub completeness_receipts: Vec<IcebergTableCommitReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProductionIcebergCommitTimes {
    pub planned_at: String,
    pub committed_at: String,
}

impl ProductionIcebergCommitTimes {
    #[must_use]
    pub fn new(planned_at: impl Into<String>, committed_at: impl Into<String>) -> Self {
        Self {
            planned_at: planned_at.into(),
            committed_at: committed_at.into(),
        }
    }
}
