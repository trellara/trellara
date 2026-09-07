use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("stream spill file {path} has corrupt JSON at line {line}: {source}")]
    StreamSpillCorrupt {
        path: String,
        line: usize,
        source: serde_json::Error,
    },
    #[error("protocol error: {0}")]
    Protocol(#[from] trellara_protocol::ProtocolError),
    #[error("invalid capture config: {0}")]
    InvalidConfig(String),
    #[error("received change outside an open transaction")]
    ChangeOutsideTransaction,
    #[error("received nested begin for transaction {0}")]
    NestedBegin(String),
    #[error("commit without an open transaction")]
    CommitWithoutBegin,
    #[error(
        "stream commit for transaction {transaction_id} arrived before stream stop for active transaction {active_transaction_id}"
    )]
    StreamCommitBeforeStop {
        transaction_id: String,
        active_transaction_id: String,
    },
    #[error(
        "stream start for transaction {transaction_id} arrived before stream stop for active transaction {active_transaction_id}"
    )]
    StreamStartBeforeStop {
        transaction_id: String,
        active_transaction_id: String,
    },
    #[error("stream stop arrived without an active streamed transaction")]
    StreamStopWithoutStart,
    #[error("invalid transaction boundary for {transaction_id}: {reason}")]
    InvalidTransactionBoundary {
        transaction_id: String,
        reason: String,
    },
    #[error("transaction {transaction_id} has more than {max_supported_changes} changes; split or stream with stricter chunk boundaries before capture can assign stable order ids")]
    TransactionOrderOverflow {
        transaction_id: String,
        max_supported_changes: u32,
    },
    #[error("transaction {transaction_id} has duplicate event total_order {total_order}")]
    DuplicateTransactionEventOrder {
        transaction_id: String,
        total_order: u32,
    },
    #[error("replication protocol error: {0}")]
    ReplicationProtocol(String),
    #[error("pgoutput parse error: {0}")]
    PgOutputParse(String),
    #[error(
        "pgoutput relation schema changed for {relation}: previous fingerprint {previous_fingerprint}, new fingerprint {new_fingerprint}; stop capture, run trellara contract test, and resume after a fresh schema handoff"
    )]
    PgOutputSchemaChanged {
        relation: String,
        previous_fingerprint: u64,
        new_fingerprint: u64,
    },
    #[error("missing pgoutput relation metadata for relation OID {relation_oid}; cannot emit schema-bound transaction envelope")]
    MissingRelationSchemaVersion { relation_oid: u32 },
    #[error("test_decoding parse error: {0}")]
    TestDecodingParse(String),
    #[error(
        "replication slot {slot_name} uses plugin {actual_plugin}, expected {expected_plugin}"
    )]
    SlotPluginMismatch {
        slot_name: String,
        expected_plugin: String,
        actual_plugin: String,
    },
    #[error("capture preflight failed: {}", .issues.join("; "))]
    PreflightFailed { issues: Vec<String> },
}

pub(crate) fn protocol_byte_label(byte: u8) -> String {
    if byte.is_ascii_graphic() {
        format!("{} (0x{byte:02X})", byte as char)
    } else {
        format!("0x{byte:02X}")
    }
}
