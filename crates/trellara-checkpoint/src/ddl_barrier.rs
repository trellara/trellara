use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    ddl_barrier_identity::{
        validate_summary_lookup_identity, DdlBarrierAckKey, DdlBarrierKey, DdlBarrierLookup,
    },
    normalize_ddl_barrier, normalize_ddl_barrier_ack, CheckpointError, DdlBarrierSummary,
    InMemoryCheckpointStore, Result,
};

pub const DDL_BARRIER_CDC_TRANSACTION_BOUNDARY: &str = "source commit LSN is the DDL barrier";

#[async_trait]
pub trait DdlBarrierStore: Send + Sync {
    async fn record_ddl_barrier(&self, barrier: DdlBarrier) -> Result<()>;
    async fn record_ddl_barrier_ack(&self, ack: DdlBarrierAck) -> Result<()>;
    async fn ddl_barrier_summary(
        &self,
        lookup: &DdlBarrierLookup,
        barrier_id: &str,
    ) -> Result<Option<DdlBarrierSummary>>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DdlBarrier {
    pub source_id: String,
    #[serde(default)]
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub barrier_lsn: String,
    pub schema_version: String,
    pub cdc_transaction_boundary: String,
    pub required_sinks: Vec<String>,
    pub requires_global_partition_pause: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DdlBarrierAck {
    pub source_id: String,
    #[serde(default)]
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub sink: String,
    pub ack_lsn: String,
    pub schema_version: String,
    pub accepted: bool,
    pub detail: String,
}

#[async_trait]
impl DdlBarrierStore for InMemoryCheckpointStore {
    async fn record_ddl_barrier(&self, barrier: DdlBarrier) -> Result<()> {
        let barrier = normalize_ddl_barrier(barrier)?;
        self.ddl_barriers
            .write()
            .await
            .insert(DdlBarrierKey::from_barrier(&barrier), barrier);
        Ok(())
    }

    async fn record_ddl_barrier_ack(&self, ack: DdlBarrierAck) -> Result<()> {
        let ack = normalize_ddl_barrier_ack(ack)?;
        let key = DdlBarrierAckKey::from_ack(&ack);
        let mut acks = self.ddl_barrier_acks.write().await;
        reject_ack_rewind(acks.get(&key), &ack)?;
        acks.insert(key, ack);
        Ok(())
    }

    async fn ddl_barrier_summary(
        &self,
        lookup: &DdlBarrierLookup,
        barrier_id: &str,
    ) -> Result<Option<DdlBarrierSummary>> {
        validate_summary_lookup_identity(lookup, barrier_id)?;
        let key = DdlBarrierKey::from_lookup(lookup, barrier_id);
        let Some(barrier) = self.ddl_barriers.read().await.get(&key).cloned() else {
            return Ok(None);
        };
        let acks = self
            .ddl_barrier_acks
            .read()
            .await
            .values()
            .filter(|ack| {
                ack.source_id == lookup.source_id
                    && ack.database_id == lookup.database_id
                    && ack.dataset_id == lookup.dataset_id
                    && ack.barrier_id == barrier_id
            })
            .cloned()
            .collect();
        Ok(Some(DdlBarrierSummary::try_from_barrier_and_acks(
            barrier, acks,
        )?))
    }
}

fn reject_ack_rewind(existing: Option<&DdlBarrierAck>, incoming: &DdlBarrierAck) -> Result<()> {
    let Some(existing) = existing else {
        return Ok(());
    };
    let incoming_lsn = crate::parse_lsn(&incoming.ack_lsn);
    let existing_lsn = crate::parse_lsn(&existing.ack_lsn);
    if incoming_lsn > existing_lsn {
        return Ok(());
    }
    if incoming_lsn == existing_lsn {
        return reject_conflicting_same_lsn_ack(existing, incoming);
    }
    Err(CheckpointError::Store(format!(
        "DDL barrier {} ack for {} cannot move backward from {} to {}",
        incoming.barrier_id, incoming.sink, existing.ack_lsn, incoming.ack_lsn
    )))
}

fn reject_conflicting_same_lsn_ack(
    existing: &DdlBarrierAck,
    incoming: &DdlBarrierAck,
) -> Result<()> {
    if existing.schema_version == incoming.schema_version && existing.accepted == incoming.accepted
    {
        return Ok(());
    }
    Err(CheckpointError::Store(format!(
        "DDL barrier {} ack for {} at {} conflicts with existing ACK decision or schema version",
        incoming.barrier_id, incoming.sink, incoming.ack_lsn
    )))
}
