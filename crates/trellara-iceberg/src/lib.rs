//! Apache Iceberg catalog integration for Trellara lake epochs.
//!
//! The default feature exposes a PostgreSQL- and catalog-independent commit
//! contract. Enable `iceberg-rust` to execute table appends through the Apache
//! Iceberg Rust catalog API.

#[cfg(feature = "production-writer")]
mod catalog_factory;
mod checkpoint_bridge;
mod checkpoint_commit;
mod checkpoint_preflight;
mod checkpoint_preflight_evidence;
mod checkpoint_preflight_types;
mod config;
mod epoch_metadata;
mod epoch_metadata_schema;
mod epoch_metadata_validation;
mod epoch_readiness;
mod epoch_readiness_evidence;
mod error;
mod fanin_l1;
mod fanin_l1_types;
#[cfg(feature = "production-writer")]
mod maintenance;
#[cfg(feature = "production-writer")]
mod metadata_commit;
mod metadata_table_specs;
#[cfg(feature = "production-writer")]
mod metadata_writer;
mod object_store;
mod planner;
mod planner_boundary;
mod planner_digest;
mod planner_files;
mod planner_mappings;
mod planner_table;
#[cfg(feature = "production-writer")]
mod production_writer;
mod provisioning;
mod provisioning_types;
mod raw_cdc_schema;
mod raw_cdc_table_spec;
mod raw_cdc_types;
#[cfg(feature = "production-writer")]
mod raw_writer;
mod receipt;
#[cfg(feature = "iceberg-rust")]
mod runtime;
#[cfg(feature = "iceberg-rust")]
mod runtime_support;
mod types;
mod writer_contract;

#[cfg(feature = "production-writer")]
pub use catalog_factory::build_iceberg_rest_catalog;
pub use checkpoint_bridge::{
    iceberg_commit_intents_from_plan, iceberg_commit_receipt_checkpoint_row,
};
pub use checkpoint_commit::commit_iceberg_epoch_with_checkpoint;
pub use checkpoint_preflight::plan_iceberg_checkpoint_preflight;
pub use checkpoint_preflight_types::{IcebergPreflightAction, IcebergPreflightDecision};
pub use config::{
    IcebergCommitConfig, IcebergRestCatalogConfig, IcebergS3ObjectStoreConfig,
    IcebergTableIdentifier, IcebergTableMapping,
};
pub use epoch_metadata::{
    plan_iceberg_epoch_metadata_append, plan_iceberg_epoch_metadata_table_spec,
};
pub use epoch_readiness::{
    verify_iceberg_epoch_checkpoint_readiness, verify_iceberg_epoch_checkpoint_store_readiness,
    IcebergEpochReadinessReport, IcebergTableReadinessEvidence, IcebergTableReadinessStatus,
};
pub use error::{IcebergIntegrationError, Result};
pub use fanin_l1::plan_iceberg_fanin_l1_table_specs;
pub use fanin_l1_types::{
    IcebergEpochMetadataAppendPlan, IcebergEpochMetadataTableSpec, IcebergFaninL1TableSpecs,
};
#[cfg(feature = "production-writer")]
pub use maintenance::{
    build_maintenance_plan, cleanup_orphan_objects, plan_iceberg_table_maintenance,
    IcebergMaintenanceFile, IcebergMaintenancePlanStatus, IcebergMaintenancePolicy,
    IcebergOrphanCleanupReport, IcebergOrphanCleanupRequest, IcebergTableMaintenancePlan,
};
#[cfg(feature = "production-writer")]
pub use metadata_commit::{plan_iceberg_metadata_commit_bundle, IcebergMetadataCommitBundle};
pub use metadata_table_specs::{
    plan_iceberg_metadata_table_specs, IcebergMetadataTableKind, IcebergMetadataTableSpec,
};
#[cfg(feature = "production-writer")]
pub use metadata_writer::{
    encode_iceberg_metadata_files, upload_iceberg_metadata_file, IcebergEncodedMetadataFile,
    IcebergUploadedMetadataFile,
};
#[cfg(feature = "production-writer")]
pub use object_store::OpenDalIcebergObjectStore;
pub use object_store::{
    upload_immutable_object, IcebergImmutableUploadProof, IcebergImmutableUploadStatus,
    IcebergObjectMetadata, IcebergObjectStore, IcebergObjectStoreError,
    IcebergObjectStoreErrorKind,
};
pub use planner::plan_iceberg_epoch_commit;
#[cfg(feature = "production-writer")]
pub use production_writer::{
    write_production_iceberg_epoch, ProductionIcebergCommitTimes, ProductionIcebergEpochResult,
};
pub use provisioning::plan_raw_cdc_iceberg_table_provisioning;
#[cfg(feature = "iceberg-rust")]
pub use provisioning::{provision_iceberg_metadata_tables, provision_raw_cdc_iceberg_tables};
pub use provisioning_types::{
    IcebergDdlAcknowledgement, IcebergRawCdcTableProvisioningPlan, IcebergTableProvisioningAction,
    IcebergTableProvisioningOutcome, IcebergTableProvisioningStatus,
};
pub use raw_cdc_table_spec::plan_raw_cdc_iceberg_table_specs;
pub use raw_cdc_types::{
    IcebergRawCdcColumn, IcebergRawCdcColumnType, IcebergRawCdcPartitionField,
    IcebergRawCdcTableSpec,
};
#[cfg(feature = "production-writer")]
pub use raw_writer::{write_and_upload_raw_cdc_file, IcebergUploadedRawDataFile};
pub use receipt::iceberg_epoch_commit_summary;
#[cfg(feature = "iceberg-rust")]
pub use runtime::commit_iceberg_table_append;
pub use types::{
    IcebergCommitRequirements, IcebergCompletedDataFile, IcebergEpochCommitPlan,
    IcebergEpochCommitSummary, IcebergFileFormat, IcebergTableAppendPlan,
    IcebergTableCommitReceipt, IcebergTableCommitStatus,
};
pub use writer_contract::{
    IcebergCatalogContract, IcebergObjectStoreContract, IcebergWriteMode, IcebergWriterContract,
    ICEBERG_WRITER_CONTRACT_VERSION,
};

pub const SNAPSHOT_PROPERTY_EPOCH_COMMIT_ID: &str = "trellara.epoch-commit-id";
pub const SNAPSHOT_PROPERTY_TABLE_COMMIT_ID: &str = "trellara.table-commit-id";
pub const SNAPSHOT_PROPERTY_DATASET_ID: &str = "trellara.dataset-id";
pub const SNAPSHOT_PROPERTY_EPOCH_ID: &str = "trellara.epoch-id";
pub const SNAPSHOT_PROPERTY_MANIFEST_DIGEST: &str = "trellara.manifest-digest";
pub const SNAPSHOT_PROPERTY_CHECKSUM_ROLLUP: &str = "trellara.checksum-rollup";
pub const SNAPSHOT_PROPERTY_TRANSACTION_COUNT: &str = "trellara.transaction-count";
pub const SNAPSHOT_PROPERTY_CHANGE_COUNT: &str = "trellara.change-count";
pub const SNAPSHOT_PROPERTY_FILE_COUNT: &str = "trellara.file-count";
pub const SNAPSHOT_PROPERTY_RECORD_COUNT: &str = "trellara.record-count";
pub const SNAPSHOT_PROPERTY_METADATA_KIND: &str = "trellara.metadata-kind";
pub const SNAPSHOT_PROPERTY_RAW_SNAPSHOT_IDS: &str = "trellara.raw-snapshot-ids";

pub const ICEBERG_EPOCH_METADATA_RELATION: &str = "trellara.fanin";
pub const ICEBERG_EPOCH_METADATA_KIND: &str = "epoch_completeness";

#[cfg(test)]
mod tests;
