use crate::postgres::PostgresCheckpointStore;
use crate::postgres_snapshot_sql::{
    LIST_SNAPSHOT_TABLE_PROGRESS, LOAD_SNAPSHOT_TABLE_PROGRESS, UPSERT_SNAPSHOT_TABLE_PROGRESS,
};
use crate::snapshot::SnapshotTableProgress;
use crate::snapshot_lookup_validation::{
    validate_snapshot_table_progress_list_lookup, validate_snapshot_table_progress_lookup,
};
use crate::snapshot_row::snapshot_table_progress_from_row;
use crate::snapshot_validation::{
    validate_snapshot_table_progress_record, validate_snapshot_table_progress_update,
};
use crate::types::FlowKey;
use crate::Result;

impl PostgresCheckpointStore {
    pub async fn record_snapshot_table_progress(
        &self,
        progress: SnapshotTableProgress,
    ) -> Result<()> {
        validate_snapshot_table_progress_record(&progress)?;
        if let Some(current) = self
            .load_snapshot_table_progress(
                &FlowKey::new(&progress.source_id, &progress.dataset_id),
                &progress.run_id,
                &progress.relation,
            )
            .await?
        {
            validate_snapshot_table_progress_update(&current, &progress)?;
        }

        self.client
            .execute(
                UPSERT_SNAPSHOT_TABLE_PROGRESS,
                &[
                    &progress.source_id,
                    &progress.dataset_id,
                    &progress.run_id,
                    &progress.relation,
                    &progress.state.to_string(),
                    &progress.copied_rows,
                    &progress.watermark_lsn,
                ],
            )
            .await?;
        Ok(())
    }

    async fn load_snapshot_table_progress(
        &self,
        flow: &FlowKey,
        run_id: &str,
        relation: &str,
    ) -> Result<Option<SnapshotTableProgress>> {
        validate_snapshot_table_progress_lookup(flow, run_id, relation)?;
        self.client
            .query_opt(
                LOAD_SNAPSHOT_TABLE_PROGRESS,
                &[&flow.source_id, &flow.dataset_id, &run_id, &relation],
            )
            .await?
            .map(snapshot_table_progress_from_row)
            .transpose()
    }

    pub async fn list_snapshot_table_progress(
        &self,
        flow: &FlowKey,
        run_id: &str,
    ) -> Result<Vec<SnapshotTableProgress>> {
        validate_snapshot_table_progress_list_lookup(flow, run_id)?;
        self.client
            .query(
                LIST_SNAPSHOT_TABLE_PROGRESS,
                &[&flow.source_id, &flow.dataset_id, &run_id],
            )
            .await?
            .into_iter()
            .map(snapshot_table_progress_from_row)
            .collect::<Result<Vec<_>>>()
    }
}
