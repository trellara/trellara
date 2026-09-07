use crate::snapshot::{SnapshotRun, SnapshotTableProgress};
use crate::snapshot_lookup_validation::{
    validate_latest_snapshot_run_lookup, validate_snapshot_run_lookup,
    validate_snapshot_table_progress_list_lookup, validate_snapshot_table_progress_lookup,
};
use crate::snapshot_validation::{
    validate_snapshot_run_record, validate_snapshot_run_update,
    validate_snapshot_table_progress_record, validate_snapshot_table_progress_update,
};
use crate::{in_memory::SnapshotRunKey, in_memory::SnapshotTableProgressKey};
use crate::{CheckpointError, FlowKey, InMemoryCheckpointStore, Result};

impl InMemoryCheckpointStore {
    pub async fn upsert_snapshot_run(&self, run: SnapshotRun) -> Result<()> {
        validate_snapshot_run_record(&run)?;
        let key = snapshot_run_key(&run);
        self.snapshot_runs.write().await.insert(key, run);
        Ok(())
    }

    pub async fn transition_snapshot_run(&self, run: SnapshotRun) -> Result<()> {
        validate_snapshot_run_record(&run)?;
        let key = snapshot_run_key(&run);
        match self.snapshot_runs.read().await.get(&key).cloned() {
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
        Ok(self
            .snapshot_runs
            .read()
            .await
            .get(&(flow.clone(), run_id.to_string()))
            .cloned())
    }

    pub async fn load_latest_snapshot_run(&self, flow: &FlowKey) -> Result<Option<SnapshotRun>> {
        validate_latest_snapshot_run_lookup(flow)?;
        Ok(self
            .snapshot_runs
            .read()
            .await
            .values()
            .filter(|run| run.source_id == flow.source_id && run.dataset_id == flow.dataset_id)
            .max_by(|left, right| {
                left.updated_at
                    .cmp(&right.updated_at)
                    .then_with(|| left.run_id.cmp(&right.run_id))
            })
            .cloned())
    }

    pub async fn record_snapshot_table_progress(
        &self,
        progress: SnapshotTableProgress,
    ) -> Result<()> {
        validate_snapshot_table_progress_record(&progress)?;
        let key = snapshot_table_progress_key(&progress);
        if let Some(current) = self.snapshot_table_progress.read().await.get(&key).cloned() {
            validate_snapshot_table_progress_update(&current, &progress)?;
        }

        self.snapshot_table_progress
            .write()
            .await
            .insert(key, progress);
        Ok(())
    }

    pub async fn load_snapshot_table_progress(
        &self,
        flow: &FlowKey,
        run_id: &str,
        relation: &str,
    ) -> Result<Option<SnapshotTableProgress>> {
        validate_snapshot_table_progress_lookup(flow, run_id, relation)?;
        Ok(self
            .snapshot_table_progress
            .read()
            .await
            .get(&(flow.clone(), run_id.to_string(), relation.to_string()))
            .cloned())
    }

    pub async fn list_snapshot_table_progress(
        &self,
        flow: &FlowKey,
        run_id: &str,
    ) -> Result<Vec<SnapshotTableProgress>> {
        validate_snapshot_table_progress_list_lookup(flow, run_id)?;
        let mut progress = self
            .snapshot_table_progress
            .read()
            .await
            .values()
            .filter(|progress| {
                progress.source_id == flow.source_id
                    && progress.dataset_id == flow.dataset_id
                    && progress.run_id == run_id
            })
            .cloned()
            .collect::<Vec<_>>();
        progress.sort_by(|left, right| left.relation.cmp(&right.relation));
        Ok(progress)
    }
}

fn snapshot_run_key(run: &SnapshotRun) -> SnapshotRunKey {
    (
        FlowKey::new(&run.source_id, &run.dataset_id),
        run.run_id.clone(),
    )
}

fn snapshot_table_progress_key(progress: &SnapshotTableProgress) -> SnapshotTableProgressKey {
    (
        FlowKey::new(&progress.source_id, &progress.dataset_id),
        progress.run_id.clone(),
        progress.relation.clone(),
    )
}
