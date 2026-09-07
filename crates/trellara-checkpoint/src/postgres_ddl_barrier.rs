use async_trait::async_trait;

use crate::ddl_barrier::{DdlBarrier, DdlBarrierAck, DdlBarrierStore};
use crate::ddl_barrier_identity::{validate_summary_lookup_identity, DdlBarrierLookup};
use crate::postgres::PostgresCheckpointStore;
use crate::postgres_ddl_barrier_sql::{
    LOAD_DDL_BARRIER, LOAD_DDL_BARRIER_ACKS, UPSERT_DDL_BARRIER, UPSERT_DDL_BARRIER_ACK,
};
use crate::{
    normalize_ddl_barrier, normalize_ddl_barrier_ack, CheckpointError, DdlBarrierSummary, Result,
};

#[async_trait]
impl DdlBarrierStore for PostgresCheckpointStore {
    async fn record_ddl_barrier(&self, barrier: DdlBarrier) -> Result<()> {
        let barrier = normalize_ddl_barrier(barrier)?;
        self.client
            .execute(
                UPSERT_DDL_BARRIER,
                &[
                    &barrier.source_id,
                    &barrier.database_id,
                    &barrier.dataset_id,
                    &barrier.barrier_id,
                    &barrier.barrier_lsn,
                    &barrier.schema_version,
                    &barrier.cdc_transaction_boundary,
                    &barrier.required_sinks,
                    &barrier.requires_global_partition_pause,
                ],
            )
            .await?;
        Ok(())
    }

    async fn record_ddl_barrier_ack(&self, ack: DdlBarrierAck) -> Result<()> {
        let ack = normalize_ddl_barrier_ack(ack)?;
        let rows = self
            .client
            .execute(
                UPSERT_DDL_BARRIER_ACK,
                &[
                    &ack.source_id,
                    &ack.database_id,
                    &ack.dataset_id,
                    &ack.barrier_id,
                    &ack.sink,
                    &ack.ack_lsn,
                    &ack.schema_version,
                    &ack.accepted,
                    &ack.detail,
                ],
            )
            .await?;
        if rows == 0 {
            return Err(CheckpointError::Store(format!(
                "DDL barrier {} ack for {} cannot move backward",
                ack.barrier_id, ack.sink
            )));
        }
        Ok(())
    }

    async fn ddl_barrier_summary(
        &self,
        lookup: &DdlBarrierLookup,
        barrier_id: &str,
    ) -> Result<Option<DdlBarrierSummary>> {
        validate_summary_lookup_identity(lookup, barrier_id)?;
        let Some(row) = self
            .client
            .query_opt(
                LOAD_DDL_BARRIER,
                &[
                    &lookup.source_id,
                    &lookup.database_id,
                    &lookup.dataset_id,
                    &barrier_id,
                ],
            )
            .await?
        else {
            return Ok(None);
        };
        let barrier = DdlBarrier {
            source_id: row.get(0),
            database_id: row.get(1),
            dataset_id: row.get(2),
            barrier_id: row.get(3),
            barrier_lsn: row.get(4),
            schema_version: row.get(5),
            cdc_transaction_boundary: row.get(6),
            required_sinks: row.get(7),
            requires_global_partition_pause: row.get(8),
        };
        let acks = self
            .client
            .query(
                LOAD_DDL_BARRIER_ACKS,
                &[
                    &lookup.source_id,
                    &lookup.database_id,
                    &lookup.dataset_id,
                    &barrier_id,
                ],
            )
            .await?
            .into_iter()
            .map(|row| DdlBarrierAck {
                source_id: row.get(0),
                database_id: row.get(1),
                dataset_id: row.get(2),
                barrier_id: row.get(3),
                sink: row.get(4),
                ack_lsn: row.get(5),
                schema_version: row.get(6),
                accepted: row.get(7),
                detail: row.get(8),
            })
            .collect();

        Ok(Some(DdlBarrierSummary::try_from_barrier_and_acks(
            barrier, acks,
        )?))
    }
}
