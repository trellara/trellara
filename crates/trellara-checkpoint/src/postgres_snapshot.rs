use crate::postgres::PostgresCheckpointStore;
use crate::postgres_snapshot_sql::{
    LOAD_LATEST_SNAPSHOT_RUN, LOAD_SNAPSHOT_RUN, UPSERT_SNAPSHOT_RUN,
};
use crate::snapshot::SnapshotRun;
use crate::snapshot_lookup_validation::{
    validate_latest_snapshot_run_lookup, validate_snapshot_run_lookup,
};
use crate::snapshot_row::snapshot_run_from_row;
use crate::snapshot_validation::{validate_snapshot_run_record, validate_snapshot_run_update};
use crate::types::FlowKey;
use crate::{CheckpointError, Result};

impl PostgresCheckpointStore {
    pub async fn upsert_snapshot_run(&self, run: SnapshotRun) -> Result<()> {
        validate_snapshot_run_record(&run)?;
        self.client
            .execute(
                UPSERT_SNAPSHOT_RUN,
                &[
                    &run.source_id,
                    &run.dataset_id,
                    &run.run_id,
                    &run.state.to_string(),
                    &run.slot_name,
                    &run.consistent_lsn,
                    &run.current_relation,
                    &run.copied_rows,
                    &run.failure_reason,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn transition_snapshot_run(&self, run: SnapshotRun) -> Result<()> {
        validate_snapshot_run_record(&run)?;
        match self
            .load_snapshot_run(&FlowKey::new(&run.source_id, &run.dataset_id), &run.run_id)
            .await?
        {
            Some(current) => validate_snapshot_run_update(&current, &run)?,
            None if !run.state.can_start_as() => {
                return Err(CheckpointError::Store(format!(
                    "snapshot run cannot start in {} state",
                    run.state
                )));
            }
            None => {}
        }

        self.upsert_snapshot_run(run).await
    }

    pub async fn load_snapshot_run(
        &self,
        flow: &FlowKey,
        run_id: &str,
    ) -> Result<Option<SnapshotRun>> {
        validate_snapshot_run_lookup(flow, run_id)?;
        self.client
            .query_opt(
                LOAD_SNAPSHOT_RUN,
                &[&flow.source_id, &flow.dataset_id, &run_id],
            )
            .await?
            .map(snapshot_run_from_row)
            .transpose()
    }

    pub async fn load_latest_snapshot_run(&self, flow: &FlowKey) -> Result<Option<SnapshotRun>> {
        validate_latest_snapshot_run_lookup(flow)?;
        self.client
            .query_opt(
                LOAD_LATEST_SNAPSHOT_RUN,
                &[&flow.source_id, &flow.dataset_id],
            )
            .await?
            .map(snapshot_run_from_row)
            .transpose()
    }
}
