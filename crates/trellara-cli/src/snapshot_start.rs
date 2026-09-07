use trellara_checkpoint::{PostgresCheckpointStore, SnapshotRunState};

use crate::{
    snapshot_exported_run_record, snapshot_run_record, BootstrapSummary, Result, SnapshotRunDraft,
    TrellaraConfig,
};

pub(crate) struct SnapshotRunStart<'a> {
    pub(crate) target_store: &'a PostgresCheckpointStore,
    pub(crate) config: &'a TrellaraConfig,
    pub(crate) run_id: &'a str,
    pub(crate) bootstrap: &'a BootstrapSummary,
    pub(crate) consistent_lsn: &'a str,
    pub(crate) completed_progress_rows: i64,
    pub(crate) existing_run_exists: bool,
    pub(crate) force: bool,
}

pub(crate) async fn start_or_restart_snapshot_run(input: SnapshotRunStart<'_>) -> Result<()> {
    if input.force && input.existing_run_exists {
        input
            .target_store
            .transition_snapshot_run(snapshot_run_record(
                input.config,
                input.run_id,
                SnapshotRunDraft {
                    state: SnapshotRunState::FailedRecoverable,
                    slot_name: &input.bootstrap.slot,
                    consistent_lsn: Some(input.consistent_lsn.to_string()),
                    current_relation: None,
                    copied_rows: input.completed_progress_rows,
                    failure_reason: Some("operator forced snapshot restart".to_string()),
                },
            ))
            .await?;
    }

    if input.existing_run_exists && !input.force {
        return Ok(());
    }

    input
        .target_store
        .transition_snapshot_run(snapshot_run_record(
            input.config,
            input.run_id,
            SnapshotRunDraft {
                state: SnapshotRunState::SlotCreated,
                slot_name: &input.bootstrap.slot,
                consistent_lsn: Some(input.consistent_lsn.to_string()),
                current_relation: None,
                copied_rows: input.completed_progress_rows,
                failure_reason: None,
            },
        ))
        .await?;
    if let Some(exported) = snapshot_exported_run_record(
        input.config,
        input.run_id,
        input.bootstrap,
        input.consistent_lsn,
        input.completed_progress_rows,
    ) {
        input.target_store.transition_snapshot_run(exported).await?;
    }

    Ok(())
}
