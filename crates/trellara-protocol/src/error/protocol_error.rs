use thiserror::Error;

use crate::{
    InvalidChangeFieldError, PartitionKeyChangePolicy, PartitionedScaleDecision,
    TransactionBoundaryKind,
};

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("failed to encode transaction envelope: {0}")]
    Encode(#[from] prost::EncodeError),
    #[error("failed to decode transaction envelope: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("checksum mismatch: expected {expected}, actual {actual}")]
    ChecksumMismatch { expected: u64, actual: u64 },
    #[error("unsupported protocol version {actual}; expected {expected}")]
    UnsupportedProtocolVersion { actual: u32, expected: u32 },
    #[error("transaction envelope is missing required field {field}")]
    MissingEnvelopeField { field: &'static str },
    #[error("transaction envelope field {field} is invalid: {reason}")]
    InvalidEnvelopeField { field: &'static str, reason: String },
    #[error("transaction envelope begin_lsn {begin_lsn:?} is after commit_lsn {commit_lsn:?}")]
    EnvelopeLsnOrder {
        begin_lsn: String,
        commit_lsn: String,
    },
    #[error("change {total_order} belongs to transaction {actual:?}, expected envelope transaction {expected:?}")]
    ChangeTransactionMismatch {
        total_order: u32,
        expected: String,
        actual: String,
    },
    #[error(transparent)]
    InvalidChangeField(#[from] InvalidChangeFieldError),
    #[error("transaction envelope has duplicate event total_order {total_order}")]
    DuplicateTransactionEventOrder { total_order: u32 },
    #[error("DDL event {total_order} belongs to transaction {actual:?}, expected envelope transaction {expected:?}")]
    DdlTransactionMismatch {
        total_order: u32,
        expected: String,
        actual: String,
    },
    #[error("DDL event {total_order} is missing required field {field}")]
    MissingDdlEventField {
        total_order: u32,
        field: &'static str,
    },
    #[error("DDL event {total_order} is invalid: {reason}")]
    InvalidDdlEvent { total_order: u32, reason: String },
    #[error("DDL event {total_order} uses unsupported operation {operation}")]
    UnsupportedDdlOperation { total_order: u32, operation: i32 },
    #[error("schema version evidence for {relation} is invalid: {reason}")]
    InvalidSchemaVersionEvidence { relation: String, reason: String },
    #[error("transaction {transaction_id} has boundary {boundary_kind:?} ({partitioned_scale_decision}), which is not yet supported in {boundary_mode}")]
    UnsupportedDdlInManifestMode {
        transaction_id: String,
        boundary_mode: &'static str,
        boundary_kind: TransactionBoundaryKind,
        partitioned_scale_decision: PartitionedScaleDecision,
    },
    #[error("partition count must be greater than zero")]
    InvalidPartitionCount,
    #[error("strict chunk size must be greater than zero")]
    InvalidStrictChunkSize,
    #[error("transaction {transaction_id} has no changes to publish")]
    EmptyTransaction { transaction_id: String },
    #[error("transaction {transaction_id} has more than {max_supported_count} events or partitions for manifest field {field}")]
    ManifestCountOverflow {
        transaction_id: String,
        field: &'static str,
        max_supported_count: u32,
    },
    #[error("transaction manifest field {field} is invalid: {reason}")]
    InvalidManifestField { field: &'static str, reason: String },
    #[error("transaction {transaction_id} manifest event count expected {expected}, got {actual}")]
    ManifestEventCountMismatch {
        transaction_id: String,
        expected: u32,
        actual: u32,
    },
    #[error("transaction {transaction_id} affected table {index} is missing relation")]
    ManifestAffectedTableMissingRelation {
        transaction_id: String,
        index: usize,
    },
    #[error("transaction {transaction_id} manifest affected table {relation} is duplicated")]
    DuplicateManifestAffectedTable {
        transaction_id: String,
        relation: String,
    },
    #[error("transaction {transaction_id} manifest affected table {relation} has zero events")]
    ManifestAffectedTableEmpty {
        transaction_id: String,
        relation: String,
    },
    #[error("transaction {transaction_id} affected table count expected {expected}, got {actual}")]
    ManifestAffectedTableCountMismatch {
        transaction_id: String,
        expected: u32,
        actual: u32,
    },
    #[error("change {total_order} uses unsupported operation {operation}")]
    UnsupportedOperation { total_order: u32, operation: i32 },
    #[error("change {total_order} is missing row image for partition key routing")]
    MissingPartitionRowImage { total_order: u32 },
    #[error("change {total_order} is missing partition key column {column}")]
    MissingPartitionKey { total_order: u32, column: String },
    #[error("change {total_order} has null partition key column {column}")]
    NullPartitionKey { total_order: u32, column: String },
    #[error("change {total_order} column {column} has unsupported value kind {value_kind}")]
    UnsupportedValueKind {
        total_order: u32,
        column: String,
        value_kind: i32,
    },
    #[error("change {total_order} changes partition key column {column}, but policy {policy} does not allow ownership moves")]
    PartitionKeyChangeRejected {
        total_order: u32,
        column: String,
        policy: PartitionKeyChangePolicy,
    },
    #[error("missing partition chunk {partition_id} for transaction {transaction_id}")]
    MissingPartitionChunk {
        transaction_id: String,
        partition_id: u32,
    },
    #[error("partition chunk {partition_id} belongs to transaction {actual}, expected {expected}")]
    PartitionTransactionMismatch {
        partition_id: u32,
        expected: String,
        actual: String,
    },
    #[error("partition chunk {partition_id} field {field} is invalid: {reason}")]
    InvalidPartitionChunkField {
        partition_id: u32,
        field: &'static str,
        reason: String,
    },
    #[error("partition chunk {partition_id} expected {expected} events, found {actual}")]
    PartitionEventCountMismatch {
        partition_id: u32,
        expected: u32,
        actual: u32,
    },
    #[error(
        "partition chunk {partition_id} checksum mismatch: expected {expected}, actual {actual}"
    )]
    PartitionChecksumMismatch {
        partition_id: u32,
        expected: u64,
        actual: u64,
    },
    #[error(
        "partition chunk {partition_id} total_order range mismatch: expected {expected_first}..{expected_last}, actual {actual_first}..{actual_last}"
    )]
    PartitionTotalOrderRangeMismatch {
        partition_id: u32,
        expected_first: u32,
        expected_last: u32,
        actual_first: u32,
        actual_last: u32,
    },
    #[error(
        "partition chunk {partition_id} is not listed in manifest for transaction {transaction_id}"
    )]
    PartitionNotInManifest {
        transaction_id: String,
        partition_id: u32,
    },
    #[error("duplicate partition chunk {partition_id} for transaction {transaction_id}")]
    DuplicatePartitionChunk {
        transaction_id: String,
        partition_id: u32,
    },
    #[error("duplicate manifest partition {partition_id} for transaction {transaction_id}")]
    DuplicateManifestPartition {
        transaction_id: String,
        partition_id: u32,
    },
    #[error("commit marker does not match manifest for transaction {transaction_id}")]
    CommitMarkerManifestMismatch { transaction_id: String },
    #[error("commit marker field {field} is invalid: {reason}")]
    InvalidCommitMarkerField { field: &'static str, reason: String },
    #[error("invalid LSN {lsn:?}: {reason}")]
    InvalidLsn { lsn: String, reason: String },
}
