use thiserror::Error;
use trellara_protocol::{PartitionedScaleDecision, ProtocolError, TransactionBoundaryKind};

#[derive(Debug, Error)]
pub enum ApplyError {
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("checkpoint error: {0}")]
    Checkpoint(#[from] trellara_checkpoint::CheckpointError),
    #[error("unsupported operation {0}")]
    UnsupportedOperation(i32),
    #[error("change {total_order} is missing relation metadata")]
    MissingRelation { total_order: u32 },
    #[error("change {total_order} relation {relation} is missing schema version evidence")]
    MissingSchemaVersionEvidence { total_order: u32, relation: String },
    #[error("change {total_order} is missing required {image} row image")]
    MissingRowImage {
        total_order: u32,
        image: &'static str,
    },
    #[error("change {total_order} has no key columns")]
    MissingKeyColumns { total_order: u32 },
    #[error("change {total_order} key column {column} was omitted as unchanged TOAST")]
    UnchangedToastKeyColumn { total_order: u32, column: String },
    #[error("change {total_order} has no mutable columns")]
    NoMutableColumns { total_order: u32 },
    #[error("column {column} uses unsupported value kind {value_kind}")]
    UnsupportedValueKind { column: String, value_kind: i32 },
    #[error("SQL placeholder range {start}..+{count} overflowed usize")]
    PlaceholderRangeOverflow { start: usize, count: usize },
    #[error("checkpoint evidence is missing required {field}")]
    MissingCheckpointEvidence { field: &'static str },
    #[error("checkpoint boundary {field} mismatch: envelope {envelope:?}, manifest {manifest:?}")]
    CheckpointBoundaryMismatch {
        field: &'static str,
        envelope: String,
        manifest: String,
    },
    #[error("checkpoint manifest boundary mode {boundary_mode} is not supported")]
    InvalidCheckpointManifestBoundaryMode { boundary_mode: i32 },
    #[error("{operation} for change {total_order} matched no target rows")]
    NoRowsMatched {
        total_order: u32,
        operation: &'static str,
    },
    #[error("DDL apply plan {barrier_id} is blocked: {blockers:?}")]
    DdlApplyBlocked {
        barrier_id: String,
        blockers: Vec<String>,
    },
    #[error("post-DDL DML release for barrier {barrier_id} is blocked: {blockers:?}")]
    DdlDmlReleaseBlocked {
        barrier_id: String,
        blockers: Vec<String>,
    },
    #[error("transaction {transaction_id} has boundary {boundary_kind:?} ({partitioned_scale_decision}) and must use the DDL barrier apply path: {reason}")]
    DdlBarrierRequired {
        transaction_id: String,
        boundary_kind: TransactionBoundaryKind,
        partitioned_scale_decision: PartitionedScaleDecision,
        reason: String,
    },
    #[error("DDL apply plan is missing required {field}")]
    MissingDdlField { field: &'static str },
    #[error("DDL apply plan field {field} is invalid: {reason}")]
    InvalidDdlField { field: &'static str, reason: String },
    #[error(
        "DDL schema evidence for {relation} expected fingerprint {expected}, but envelope carried {actual}"
    )]
    DdlSchemaVersionMismatch {
        relation: String,
        expected: u64,
        actual: u64,
    },
    #[error("DDL ACK evidence field {field} is invalid: {reason}")]
    InvalidDdlAckEvidence { field: &'static str, reason: String },
    #[error("DDL statement {change} is not safe for automatic target apply: {reason}")]
    UnsafeDdlStatement { change: String, reason: String },
}

pub type Result<T> = std::result::Result<T, ApplyError>;
