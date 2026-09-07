use serde::{Deserialize, Serialize};

use crate::{IcebergIntegrationError, Result};

pub const ICEBERG_WRITER_CONTRACT_VERSION: u16 = 1;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergCatalogContract {
    Rest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergObjectStoreContract {
    S3Compatible,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergWriteMode {
    AppendOnlyRawCdc,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergWriterContract {
    pub contract_version: u16,
    pub catalog: IcebergCatalogContract,
    pub object_store: IcebergObjectStoreContract,
    pub write_mode: IcebergWriteMode,
    pub require_durable_upload_proof: bool,
    pub require_immutable_object_write: bool,
    pub persist_intent_before_catalog_call: bool,
    pub validate_snapshot_before_receipt: bool,
    pub publish_epoch_metadata_after_all_receipts: bool,
    pub publish_queryable_epoch_completeness_table: bool,
    pub partitioned_data_files: bool,
    pub schema_evolution_requires_ddl_ack: bool,
}

impl IcebergWriterContract {
    #[must_use]
    pub const fn production_v1() -> Self {
        Self {
            contract_version: ICEBERG_WRITER_CONTRACT_VERSION,
            catalog: IcebergCatalogContract::Rest,
            object_store: IcebergObjectStoreContract::S3Compatible,
            write_mode: IcebergWriteMode::AppendOnlyRawCdc,
            require_durable_upload_proof: true,
            require_immutable_object_write: true,
            persist_intent_before_catalog_call: true,
            validate_snapshot_before_receipt: true,
            publish_epoch_metadata_after_all_receipts: true,
            publish_queryable_epoch_completeness_table: true,
            partitioned_data_files: true,
            schema_evolution_requires_ddl_ack: true,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.contract_version != ICEBERG_WRITER_CONTRACT_VERSION {
            return Err(IcebergIntegrationError::UnsupportedWriterContractVersion {
                expected: ICEBERG_WRITER_CONTRACT_VERSION,
                actual: self.contract_version,
            });
        }
        require(
            "require_durable_upload_proof",
            self.require_durable_upload_proof,
        )?;
        require(
            "require_immutable_object_write",
            self.require_immutable_object_write,
        )?;
        require(
            "persist_intent_before_catalog_call",
            self.persist_intent_before_catalog_call,
        )?;
        require(
            "validate_snapshot_before_receipt",
            self.validate_snapshot_before_receipt,
        )?;
        require(
            "publish_epoch_metadata_after_all_receipts",
            self.publish_epoch_metadata_after_all_receipts,
        )?;
        require(
            "publish_queryable_epoch_completeness_table",
            self.publish_queryable_epoch_completeness_table,
        )?;
        require("partitioned_data_files", self.partitioned_data_files)?;
        require(
            "schema_evolution_requires_ddl_ack",
            self.schema_evolution_requires_ddl_ack,
        )
    }
}

fn require(field: &'static str, enabled: bool) -> Result<()> {
    if enabled {
        Ok(())
    } else {
        Err(IcebergIntegrationError::InvalidWriterContract {
            field,
            reason: "must be enabled for the production v1 writer",
        })
    }
}
