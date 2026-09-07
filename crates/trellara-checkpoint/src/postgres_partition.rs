use crate::partition::PartitionCheckpoint;
use crate::partition_checkpoint_row::partition_checkpoint_from_row;
use crate::postgres::PostgresCheckpointStore;
use crate::postgres_partition_sql::{LOAD_PARTITION_CHECKPOINTS, UPSERT_PARTITION_CHECKPOINT};
use crate::types::FlowKey;
use crate::validation::validate_partition_checkpoint;
use crate::{CheckpointError, Result};

impl PostgresCheckpointStore {
    pub async fn record_partition_checkpoint(&self, checkpoint: PartitionCheckpoint) -> Result<()> {
        validate_partition_checkpoint(&checkpoint)?;
        let partition_id = i32::try_from(checkpoint.partition_id).map_err(|_| {
            CheckpointError::Store(format!(
                "partition {} is too large for postgres integer storage",
                checkpoint.partition_id
            ))
        })?;
        self.client
            .execute(
                UPSERT_PARTITION_CHECKPOINT,
                &[
                    &checkpoint.source_id,
                    &checkpoint.dataset_id,
                    &partition_id,
                    &checkpoint.last_durable_lsn,
                    &checkpoint.last_applied_lsn,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn load_partition_checkpoints(
        &self,
        flow: &FlowKey,
    ) -> Result<Vec<PartitionCheckpoint>> {
        self.client
            .query(
                LOAD_PARTITION_CHECKPOINTS,
                &[&flow.source_id, &flow.dataset_id],
            )
            .await?
            .into_iter()
            .map(partition_checkpoint_from_row)
            .collect()
    }
}
