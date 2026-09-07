use trellara_checkpoint::SnapshotRunState;
use trellara_verify::{reseed_postgres_table, PostgresReseedConfig};

use crate::{
    i64_to_u64_count, record_snapshot_table_copy_completed, record_snapshot_table_copy_started,
    skipped_snapshot_table_summary, snapshot_copy_failure_records,
    snapshot_progress_complete_at_boundary, snapshot_run_record, u64_to_i64_count, Result,
    SnapshotCopyContext, SnapshotCopyTableSummary, SnapshotRunDraft, TrellaraConfig,
};

pub(crate) struct SnapshotCopyTablesOutcome {
    pub(crate) copied_rows: u64,
    pub(crate) skipped_tables: usize,
    pub(crate) tables: Vec<SnapshotCopyTableSummary>,
}

pub(crate) async fn copy_snapshot_tables(
    config: &TrellaraConfig,
    target_database_url: &str,
    context: &mut SnapshotCopyContext<'_>,
    force: bool,
) -> Result<SnapshotCopyTablesOutcome> {
    let mut tables = Vec::new();
    let mut copied_rows = 0_u64;
    let mut skipped_tables = 0_usize;

    for table in context.plan.selected_tables.iter().copied() {
        let relation = table.relation_id();
        let relation_name = relation.display_name();
        if !force
            && context
                .planned_progress
                .get(&relation_name)
                .is_some_and(|progress| {
                    snapshot_progress_complete_at_boundary(progress, &context.consistent_lsn)
                })
        {
            let progress = context
                .planned_progress
                .get(&relation_name)
                .expect("checked completed progress");
            skipped_tables += 1;
            copied_rows += i64_to_u64_count("snapshot copied_rows", progress.copied_rows)?;
            tables.push(skipped_snapshot_table_summary(relation_name, progress)?);
            continue;
        }

        context
            .target_store
            .transition_snapshot_run(snapshot_run_record(
                config,
                &context.run_id,
                SnapshotRunDraft {
                    state: SnapshotRunState::CopyingTable,
                    slot_name: &context.bootstrap.slot,
                    consistent_lsn: Some(context.consistent_lsn.clone()),
                    current_relation: Some(relation_name.clone()),
                    copied_rows: u64_to_i64_count("snapshot copied_rows", copied_rows)?,
                    failure_reason: None,
                },
            ))
            .await?;
        record_snapshot_table_copy_started(
            &context.target_store,
            config,
            &context.run_id,
            &relation_name,
            &context.consistent_lsn,
        )
        .await?;

        let verify = table.verify.as_ref().cloned().unwrap_or_default();
        let contract = table.contract.as_ref().cloned().unwrap_or_default();
        let summary = match reseed_postgres_table(PostgresReseedConfig {
            source_database_url: config.source.database_url.expose().to_string(),
            target_database_url: target_database_url.to_string(),
            relation,
            primary_key: verify.primary_key,
            excluded_columns: verify.excluded_columns,
            target_owned_columns: contract.target_owned_columns,
            row_filter: verify.row_filter,
            watermark_lsn: context.consistent_lsn.clone(),
            source_snapshot_name: context.bootstrap.exported_snapshot_name.clone(),
        })
        .await
        {
            Ok(summary) => summary,
            Err(error) => {
                let (failed_run, failed_progress) = snapshot_copy_failure_records(
                    config,
                    &context.run_id,
                    &context.bootstrap.slot,
                    &context.consistent_lsn,
                    &relation_name,
                    u64_to_i64_count("snapshot copied_rows", copied_rows)?,
                    error.to_string(),
                );
                context
                    .target_store
                    .transition_snapshot_run(failed_run)
                    .await?;
                context
                    .target_store
                    .record_snapshot_table_progress(failed_progress)
                    .await?;
                return Err(error.into());
            }
        };
        copied_rows += summary.copied_rows;
        let completed_progress = record_snapshot_table_copy_completed(
            &context.target_store,
            config,
            &context.run_id,
            &relation_name,
            &context.consistent_lsn,
            summary.copied_rows,
        )
        .await?;
        context
            .planned_progress
            .insert(relation_name.clone(), completed_progress);
        tables.push(SnapshotCopyTableSummary {
            relation: relation_name,
            state: SnapshotRunState::CopyComplete.to_string(),
            copied_rows: summary.copied_rows,
            skipped: false,
            watermark_lsn: context.consistent_lsn.clone(),
        });
    }

    Ok(SnapshotCopyTablesOutcome {
        copied_rows,
        skipped_tables,
        tables,
    })
}
