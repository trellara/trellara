use std::collections::HashMap;

use trellara_checkpoint::{
    FlowKey, PostgresCheckpointStore, SnapshotRunState, SnapshotTableProgress,
};

use crate::{
    bootstrap_configured_snapshot_capture, completed_snapshot_copy_summary,
    default_snapshot_run_id, require_non_empty, snapshot_progress_by_relation, BootstrapSummary,
    Result, SnapshotCopyPlan, SnapshotCopySummary, TrellaraConfig,
};

pub(crate) enum SnapshotCopySetup<'a> {
    Completed(Box<SnapshotCopySummary>),
    Ready(Box<SnapshotCopyContext<'a>>),
}

pub(crate) struct SnapshotCopyContext<'a> {
    pub(crate) run_id: String,
    pub(crate) plan: SnapshotCopyPlan<'a>,
    pub(crate) target_store: PostgresCheckpointStore,
    pub(crate) planned_progress: HashMap<String, SnapshotTableProgress>,
    pub(crate) existing_run_exists: bool,
    pub(crate) bootstrap: BootstrapSummary,
    pub(crate) consistent_lsn: String,
    pub(crate) _exported_slot_holder: Option<trellara_pg_capture::ExportedLogicalSlot>,
}

impl SnapshotCopyContext<'_> {
    pub(crate) fn completed_progress_rows(&self) -> i64 {
        self.planned_progress
            .values()
            .filter(|progress| progress.state == SnapshotRunState::CopyComplete)
            .map(|progress| progress.copied_rows)
            .sum()
    }
}

pub(crate) async fn prepare_snapshot_copy<'a>(
    config: &'a TrellaraConfig,
    target_database_url: &str,
    run_id: Option<&str>,
    table_filter: Option<&str>,
    create_if_missing: bool,
    force: bool,
) -> Result<SnapshotCopySetup<'a>> {
    config.validate()?;
    let run_id = run_id
        .map(str::to_string)
        .unwrap_or_else(default_snapshot_run_id);
    require_non_empty("snapshot run_id", &run_id)?;
    let plan = SnapshotCopyPlan::from_tables(&config.dataset.tables, table_filter)?;
    let target_store = PostgresCheckpointStore::connect(target_database_url, true).await?;
    let flow = FlowKey::new(&config.source.id, &config.dataset.id);
    let planned_progress = snapshot_progress_by_relation(
        target_store
            .list_snapshot_table_progress(&flow, &run_id)
            .await?,
    );
    let existing_run = target_store.load_snapshot_run(&flow, &run_id).await?;
    if !force {
        if let Some(summary) = completed_snapshot_copy_summary(
            config,
            &run_id,
            &plan,
            existing_run.as_ref(),
            &planned_progress,
        ) {
            return Ok(SnapshotCopySetup::Completed(Box::new(summary)));
        }
    }

    let source_store = PostgresCheckpointStore::connect(&config.source.database_url, true).await?;
    let source_checkpoint = trellara_relay::load_source_checkpoint(
        &source_store,
        &config.source.id,
        &config.dataset.id,
    )
    .await?;
    let watermark_lsn = source_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.last_durable_lsn.clone())
        .unwrap_or_default();
    let (bootstrap, exported_slot_holder) =
        bootstrap_configured_snapshot_capture(config, create_if_missing).await?;
    let consistent_lsn = bootstrap.consistent_lsn.clone().unwrap_or(watermark_lsn);

    Ok(SnapshotCopySetup::Ready(Box::new(SnapshotCopyContext {
        run_id,
        plan,
        target_store,
        planned_progress,
        existing_run_exists: existing_run.is_some(),
        bootstrap,
        consistent_lsn,
        _exported_slot_holder: exported_slot_holder,
    })))
}
