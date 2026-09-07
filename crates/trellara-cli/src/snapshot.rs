use crate::{
    complete_snapshot_copy_run, copy_snapshot_tables, prepare_snapshot_copy, snapshot_copy_summary,
    snapshot_handoff_recovery_for_flow, start_or_restart_snapshot_run, Result,
    SnapshotCopyCompletion, SnapshotCopySetup, SnapshotCopySummary, SnapshotCopySummaryDraft,
    SnapshotRunStart, TrellaraConfig,
};

pub(crate) async fn snapshot_configured_tables(
    config: &TrellaraConfig,
    target_database_url: &str,
    run_id: Option<&str>,
    table_filter: Option<&str>,
    create_if_missing: bool,
    force: bool,
) -> Result<SnapshotCopySummary> {
    let context = match prepare_snapshot_copy(
        config,
        target_database_url,
        run_id,
        table_filter,
        create_if_missing,
        force,
    )
    .await?
    {
        SnapshotCopySetup::Completed(summary) => return Ok(*summary),
        SnapshotCopySetup::Ready(context) => *context,
    };
    let completed_progress_rows = context.completed_progress_rows();
    let mut context = context;

    start_or_restart_snapshot_run(SnapshotRunStart {
        target_store: &context.target_store,
        config,
        run_id: &context.run_id,
        bootstrap: &context.bootstrap,
        consistent_lsn: &context.consistent_lsn,
        completed_progress_rows,
        existing_run_exists: context.existing_run_exists,
        force,
    })
    .await?;

    let copy_outcome =
        copy_snapshot_tables(config, target_database_url, &mut context, force).await?;

    let (handoff_ready, _final_run_copied_rows) = complete_snapshot_copy_run(
        &context.target_store,
        SnapshotCopyCompletion {
            config,
            run_id: &context.run_id,
            slot: &context.bootstrap.slot,
            consistent_lsn: &context.consistent_lsn,
            handoff_relations: &context.plan.handoff_relations,
            planned_progress: &context.planned_progress,
            copied_rows: copy_outcome.copied_rows,
        },
    )
    .await?;
    let (handoff_blocker_codes, recovery_actions) = snapshot_handoff_recovery_for_flow(
        &config.source.id,
        &config.dataset.id,
        &context.run_id,
        &context.plan.handoff_relations,
        &context.planned_progress,
        &context.consistent_lsn,
    );

    Ok(snapshot_copy_summary(
        config,
        SnapshotCopySummaryDraft {
            run_id: context.run_id,
            slot: context.bootstrap.slot,
            consistent_lsn: context.consistent_lsn,
            handoff_ready,
            table_count: copy_outcome.tables.len(),
            skipped_table_count: copy_outcome.skipped_tables,
            copied_rows: copy_outcome.copied_rows,
            tables: copy_outcome.tables,
            exported_snapshot_name: context.bootstrap.exported_snapshot_name,
            handoff_blocker_codes,
            recovery_actions,
        },
    ))
}
