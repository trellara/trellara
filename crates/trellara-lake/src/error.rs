#[derive(Debug, thiserror::Error)]
pub enum LakeError {
    #[error("invalid transaction envelope: {0}")]
    InvalidEnvelope(#[from] trellara_protocol::ProtocolError),
    #[error("change {total_order} is missing relation")]
    MissingRelation { total_order: u32 },
    #[error("relation {relation} is not configured for lake materialization")]
    UnknownRelation { relation: String },
    #[error("change {total_order} is missing row image required for {materialization}")]
    MissingRowImage {
        total_order: u32,
        materialization: &'static str,
    },
    #[error("change {total_order} is missing primary key column {primary_key}")]
    MissingPrimaryKey {
        total_order: u32,
        primary_key: String,
    },
    #[error("change {total_order} uses unsupported operation {operation}")]
    UnsupportedOperation { total_order: u32, operation: i32 },
    #[error("column {column} uses unsupported value kind {value_kind}")]
    UnsupportedValueKind { column: String, value_kind: i32 },
    #[error("partition manifest expected {expected} events, found {actual}")]
    ManifestEventCountMismatch { expected: u32, actual: u32 },
    #[error("partition manifest {field} mismatch: envelope {envelope:?}, manifest {manifest:?}")]
    ManifestBoundaryMismatch {
        field: &'static str,
        envelope: String,
        manifest: String,
    },
    #[error("partition manifest field {field} exceeds supported count {max_supported_count}")]
    ManifestCountOverflow {
        field: &'static str,
        max_supported_count: u32,
    },
    #[error(
        "partition manifest partition {partition_id} {field}={total_order} is missing from the reconstructed envelope"
    )]
    ManifestOrderBoundaryMissing {
        partition_id: u32,
        field: &'static str,
        total_order: u32,
    },
    #[error("partition manifest boundary mode {boundary_mode} is not supported")]
    InvalidManifestBoundaryMode { boundary_mode: i32 },
    #[error("idempotency key {idempotency_key} appeared with conflicting transaction evidence")]
    ConflictingDuplicate { idempotency_key: String },
    #[error("raw CDC writer source_bucket_count must be greater than zero")]
    InvalidSourceBucketCount,
    #[error("raw CDC DDL event for {relation} is missing matching schema version evidence")]
    MissingRawCdcDdlSchemaVersion { relation: String },
    #[error(
        "raw CDC DDL event for {relation} has schema version {actual}, expected final DDL fingerprint {expected}"
    )]
    RawCdcDdlSchemaVersionMismatch {
        relation: String,
        expected: u64,
        actual: u64,
    },
    #[error("raw CDC count {field} overflowed usize")]
    RawCdcCountOverflow { field: &'static str },
    #[error("raw CDC Parquet count {field} overflowed its supported representation")]
    RawCdcParquetCountOverflow { field: &'static str },
    #[error("raw CDC Parquet file {field} mismatch: expected {expected}, found {actual}")]
    RawCdcParquetBoundaryMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("raw CDC Parquet {operation} failed: {message}")]
    RawCdcParquet {
        operation: &'static str,
        message: String,
    },
    #[error("lake epoch count {field} overflowed usize")]
    EpochCountOverflow { field: &'static str },
    #[error("lake epoch source window for {source_id} was provided more than once")]
    DuplicateEpochSourceWindow { source_id: String },
    #[error("raw CDC epoch metadata has duplicate source row {source_id} for epoch {epoch_id}")]
    DuplicateRawCdcEpochSource { epoch_id: String, source_id: String },
    #[error(
        "raw CDC epoch metadata {row_kind} row {row_id} belongs to epoch {row_epoch_id}, expected {epoch_id}"
    )]
    RawCdcEpochMetadataBoundaryMismatch {
        row_kind: &'static str,
        row_id: String,
        row_epoch_id: String,
        epoch_id: String,
    },
    #[error("raw CDC epoch metadata {field} is invalid: {reason}")]
    InvalidRawCdcEpochMetadataField { field: &'static str, reason: String },
    #[error("raw CDC epoch metadata has duplicate table row {relation} for epoch {epoch_id}")]
    DuplicateRawCdcEpochTable { epoch_id: String, relation: String },
    #[error(
        "raw CDC epoch metadata has duplicate partition row source={source_id} partition={partition_id} for epoch {epoch_id}"
    )]
    DuplicateRawCdcEpochPartition {
        epoch_id: String,
        source_id: String,
        partition_id: u32,
    },
    #[error(
        "raw CDC epoch metadata source {field} rollup mismatch for epoch {epoch_id}: expected {expected}, found {actual}"
    )]
    RawCdcEpochSourceRollupMismatch {
        epoch_id: String,
        field: &'static str,
        expected: u128,
        actual: u128,
    },
    #[error(
        "raw CDC epoch metadata table {field} rollup mismatch for epoch {epoch_id}: expected {expected}, found {actual}"
    )]
    RawCdcEpochTableRollupMismatch {
        epoch_id: String,
        field: &'static str,
        expected: u128,
        actual: u128,
    },
    #[error(
        "raw CDC epoch metadata partition {field} rollup mismatch for epoch {epoch_id}: expected {expected}, found {actual}"
    )]
    RawCdcEpochPartitionRollupMismatch {
        epoch_id: String,
        field: &'static str,
        expected: u128,
        actual: u128,
    },
    #[error("lake commit_lsn {commit_lsn} must be a non-zero PostgreSQL LSN like 0/16B9000")]
    InvalidCommitLsn { commit_lsn: String },
    #[error("DDL ack evidence is missing {field}")]
    MissingDdlAckField { field: &'static str },
    #[error("DDL ack_lsn {ack_lsn} must be a non-zero PostgreSQL LSN like 0/16B9000")]
    InvalidDdlAckLsn { ack_lsn: String },
    #[error("DDL ack evidence field {field} is invalid: {reason}")]
    InvalidDdlAckField { field: &'static str, reason: String },
    #[error(
        "lake epoch verification boundary mismatch: epoch {epoch_id}, verification {verification_epoch_id}"
    )]
    EpochVerificationBoundaryMismatch {
        epoch_id: String,
        verification_epoch_id: String,
    },
    #[error("lake epoch {epoch_id} verification status {verification_status:?} is not consumable")]
    EpochVerificationNotMatched {
        epoch_id: String,
        verification_status: crate::LakeEpochVerificationStatus,
    },
    #[error("lake epoch {epoch_id} is complete_with_gaps and requires explicit gap acceptance")]
    EpochRequiresGapAcceptance { epoch_id: String },
    #[error("lake epoch {epoch_id} state {state:?} is not consumable")]
    EpochNotConsumable {
        epoch_id: String,
        state: crate::LakeCompletenessState,
    },
    #[error(
        "lake epoch {epoch_id} source counts are inconsistent: required={required_source_count}, complete={complete_source_count}, missing={missing_source_count}, quarantined={quarantined_source_count}"
    )]
    EpochSourceCountMismatch {
        epoch_id: String,
        required_source_count: usize,
        complete_source_count: usize,
        missing_source_count: usize,
        quarantined_source_count: usize,
    },
    #[error(
        "lake epoch {epoch_id} state {state:?} is inconsistent with missing={missing_source_count} quarantined={quarantined_source_count}"
    )]
    EpochCompletenessStateMismatch {
        epoch_id: String,
        state: crate::LakeCompletenessState,
        missing_source_count: usize,
        quarantined_source_count: usize,
    },
    #[error(
        "lake epoch {epoch_id} verification {field} mismatch: epoch={epoch_count}, input={input_count}, lake={lake_count}"
    )]
    EpochVerificationCountMismatch {
        epoch_id: String,
        field: &'static str,
        epoch_count: usize,
        input_count: usize,
        lake_count: usize,
    },
}
