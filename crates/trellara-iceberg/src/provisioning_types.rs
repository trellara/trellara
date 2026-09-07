use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{IcebergRawCdcColumn, IcebergRawCdcPartitionField, IcebergTableIdentifier};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergTableProvisioningAction {
    EnsureNamespace,
    CreateTableIfMissing,
    VerifyCompatible,
    EvolveSchemaAfterDdlAck,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergRawCdcTableProvisioningPlan {
    pub lake_table_name: String,
    pub relation: String,
    pub target: IcebergTableIdentifier,
    pub schema_fingerprint_sha256: String,
    pub actions: Vec<IcebergTableProvisioningAction>,
    pub columns: Vec<IcebergRawCdcColumn>,
    pub partition_fields: Vec<IcebergRawCdcPartitionField>,
    pub write_mode: String,
    pub schema_evolution_gate: String,
    pub partition_evolution_gate: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergDdlAcknowledgement {
    pub dataset_id: String,
    pub barrier_id: String,
    pub ack_lsn: String,
    pub schema_version: String,
    pub manifest_digest: String,
    pub authorized_schema_fingerprints: BTreeSet<String>,
}

impl IcebergDdlAcknowledgement {
    pub fn from_raw_cdc_lake_ack(
        evidence: &trellara_lake::RawCdcLakeDdlAckEvidence,
        schema_fingerprints: impl IntoIterator<Item = String>,
    ) -> crate::Result<Self> {
        if evidence.sink != "raw_cdc_lake" || evidence.release_gate != "post_ddl_dml_release" {
            return Err(crate::IcebergIntegrationError::DdlAcknowledgementRequired {
                target: evidence.dataset_id.clone(),
                reason: "acknowledgement is not accepted raw_cdc_lake post-DDL release evidence"
                    .to_string(),
            });
        }
        let authorized_schema_fingerprints: BTreeSet<String> =
            schema_fingerprints.into_iter().collect();
        if authorized_schema_fingerprints.is_empty()
            || authorized_schema_fingerprints.iter().any(|digest| {
                digest.len() != 64
                    || !digest
                        .chars()
                        .all(|character| character.is_ascii_hexdigit())
            })
        {
            return Err(crate::IcebergIntegrationError::DdlAcknowledgementRequired {
                target: evidence.dataset_id.clone(),
                reason: "acknowledgement must bind at least one valid schema fingerprint"
                    .to_string(),
            });
        }
        Ok(Self {
            dataset_id: evidence.dataset_id.clone(),
            barrier_id: evidence.barrier_id.clone(),
            ack_lsn: evidence.ack_lsn.clone(),
            schema_version: evidence.schema_version.clone(),
            manifest_digest: evidence.manifest_digest.clone(),
            authorized_schema_fingerprints,
        })
    }

    #[cfg(feature = "iceberg-rust")]
    pub(crate) fn authorizes(&self, dataset_id: &str, schema_fingerprint: &str) -> bool {
        self.dataset_id == dataset_id
            && self
                .authorized_schema_fingerprints
                .contains(schema_fingerprint)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergTableProvisioningStatus {
    Created,
    Compatible,
    SchemaEvolved,
    ReconciledAfterAmbiguousFailure,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableProvisioningOutcome {
    pub target: IcebergTableIdentifier,
    pub schema_fingerprint_sha256: String,
    pub status: IcebergTableProvisioningStatus,
}
