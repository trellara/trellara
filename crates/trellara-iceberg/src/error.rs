use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcebergIntegrationError {
    #[error("lake epoch is not consumable: {0}")]
    Lake(#[from] trellara_lake::LakeError),
    #[error("invalid Iceberg table identifier {identifier}: {reason}")]
    InvalidTableIdentifier { identifier: String, reason: String },
    #[error("lake table mapping is missing for {lake_table_name}")]
    MissingTableMapping { lake_table_name: String },
    #[error("raw CDC write plan field {field} mismatch: expected {expected}, found {actual}")]
    WritePlanBoundaryMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("lake table mapping for {lake_table_name} was provided more than once")]
    DuplicateTableMapping { lake_table_name: String },
    #[error("Iceberg target {target} is mapped from more than one lake table")]
    DuplicateIcebergTarget { target: String },
    #[error("completed data file {planned_object_key} was provided more than once")]
    DuplicateCompletedDataFile { planned_object_key: String },
    #[error("planned data file {planned_object_key} has no completed Parquet file")]
    MissingCompletedDataFile { planned_object_key: String },
    #[error("completed data file {planned_object_key} was not present in the Trellara write plan")]
    UnplannedCompletedDataFile { planned_object_key: String },
    #[error("completed data file {planned_object_key} field {field} mismatch: expected {expected}, found {actual}")]
    CompletedDataFileMismatch {
        planned_object_key: String,
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("completed data file {planned_object_key} field {field} is invalid: {reason}")]
    InvalidCompletedDataFile {
        planned_object_key: String,
        field: &'static str,
        reason: String,
    },
    #[error("unsupported Iceberg writer contract version {actual}; expected {expected}")]
    UnsupportedWriterContractVersion { expected: u16, actual: u16 },
    #[error("Iceberg writer contract field {field} is invalid: {reason}")]
    InvalidWriterContract {
        field: &'static str,
        reason: &'static str,
    },
    #[error("Iceberg count {field} overflowed its supported representation")]
    CountOverflow { field: &'static str },
    #[error("Iceberg commit receipt for {target} was provided more than once")]
    DuplicateCommitReceipt { target: String },
    #[error("Iceberg commit receipt references unplanned target {target}")]
    UnplannedCommitReceipt { target: String },
    #[error("Iceberg commit receipt for {target} field {field} mismatch: expected {expected}, found {actual}")]
    CommitReceiptMismatch {
        target: String,
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("Iceberg commit receipt for {target} field {field} is invalid: {reason}")]
    InvalidCommitReceipt {
        target: String,
        field: &'static str,
        reason: String,
    },
    #[error("Iceberg epoch metadata for {epoch_id} is not ready; missing raw table receipts: {missing_tables:?}")]
    EpochMetadataNotReady {
        epoch_id: String,
        missing_tables: Vec<String>,
    },
    #[error("Iceberg runtime does not yet support partitioned table {target}")]
    PartitionedTableUnsupported { target: String },
    #[error("Iceberg partition spec for {target} is unsupported: {reason}")]
    UnsupportedPartitionSpec { target: String, reason: String },
    #[error("Iceberg catalog configuration field {field} is invalid: {reason}")]
    InvalidCatalogConfig { field: &'static str, reason: String },
    #[error("Iceberg object-store configuration field {field} is invalid: {reason}")]
    InvalidObjectStoreConfig { field: &'static str, reason: String },
    #[error("Iceberg catalog {operation} failed for {target}: {message}")]
    Catalog {
        operation: &'static str,
        target: String,
        message: String,
    },
    #[error("Iceberg object-store {operation} failed for {object_key}: {message}")]
    ObjectStore {
        operation: &'static str,
        object_key: String,
        message: String,
    },
    #[error("immutable Iceberg object {object_key} conflicts with existing content: expected {expected_sha256}, found {actual_sha256}")]
    ImmutableObjectConflict {
        object_key: String,
        expected_sha256: String,
        actual_sha256: String,
    },
    #[error("Iceberg provisioning for {target} is incompatible: {reason}")]
    IncompatibleTable { target: String, reason: String },
    #[error(
        "Iceberg schema evolution for {target} is not authorized by DDL acknowledgement: {reason}"
    )]
    DdlAcknowledgementRequired { target: String, reason: String },
    #[error("Iceberg maintenance plan is unsafe for {target}: {reason}")]
    UnsafeMaintenancePlan { target: String, reason: String },
    #[error("Iceberg metadata encoding failed for {table}: {message}")]
    MetadataEncoding { table: String, message: String },
    #[error("Iceberg checkpoint {operation} failed: {message}")]
    Checkpoint {
        operation: &'static str,
        message: String,
    },
    #[error("Iceberg table {target} contains conflicting Trellara commit evidence: {reason}")]
    ConflictingCatalogEvidence { target: String, reason: String },
}

pub type Result<T> = std::result::Result<T, IcebergIntegrationError>;
