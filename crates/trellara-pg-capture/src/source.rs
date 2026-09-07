use async_trait::async_trait;
use trellara_protocol::TransactionEnvelope;

use crate::replication::ReplicationBootstrapConnection;
use crate::{CapturedRelation, Result, TablePreflight};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureBootstrap {
    pub publication_name: String,
    pub slot_name: String,
    pub consistent_lsn: Option<String>,
    pub exported_snapshot_name: Option<String>,
    pub relations: Vec<CapturedRelation>,
    pub preflight: Vec<TablePreflight>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalSlotBootstrap {
    pub created: bool,
    pub consistent_lsn: Option<String>,
}

#[derive(Debug)]
pub struct ExportedLogicalSlot {
    pub slot_name: String,
    pub consistent_lsn: String,
    pub snapshot_name: String,
    pub output_plugin: String,
    pub(crate) _holder: ReplicationBootstrapConnection,
}

#[derive(Debug)]
pub struct ExportedCaptureBootstrap {
    pub publication_name: String,
    pub exported_slot: ExportedLogicalSlot,
    pub relations: Vec<CapturedRelation>,
    pub preflight: Vec<TablePreflight>,
}

#[async_trait]
pub trait ChangeSource {
    async fn next_transaction(&mut self) -> Result<Option<TransactionEnvelope>>;

    async fn acknowledge_durable_lsn(&mut self, _lsn: &str) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
pub trait PgChangeSource: Send {
    async fn bootstrap(&mut self) -> Result<CaptureBootstrap>;
    async fn next_transaction(&mut self) -> Result<Option<TransactionEnvelope>>;
    async fn acknowledge_durable(&mut self, commit_lsn: &str) -> Result<()>;
}
