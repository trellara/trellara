use std::collections::HashMap;

use trellara_checkpoint::{SnapshotRun, SnapshotRunState, SnapshotTableProgress};

use crate::{
    i64_to_u64_count, snapshot_handoff_proof_command, snapshot_handoff_ready_for_flow,
    snapshot_progress_complete_at_boundary, SnapshotCopyPlan, SnapshotCopySummary,
    SnapshotCopyTableSummary, TrellaraConfig,
};

pub(crate) fn completed_snapshot_copy_summary(
    config: &TrellaraConfig,
    run_id: &str,
    plan: &SnapshotCopyPlan<'_>,
    existing_run: Option<&SnapshotRun>,
    existing_progress: &HashMap<String, SnapshotTableProgress>,
) -> Option<SnapshotCopySummary> {
    let existing_run = existing_run?;
    if !matches!(
        existing_run.state,
        SnapshotRunState::StreamHandoffReady
            | SnapshotRunState::Streaming
            | SnapshotRunState::Verified
    ) {
        return None;
    }
    let consistent_lsn = existing_run.consistent_lsn.clone().unwrap_or_default();
    if !snapshot_handoff_ready_for_flow(
        &config.source.id,
        &config.dataset.id,
        run_id,
        &plan.handoff_relations,
        existing_progress,
        &consistent_lsn,
    ) {
        return None;
    }

    let mut tables = Vec::new();
    let mut copied_rows = 0_u64;
    for table in &plan.selected_tables {
        let relation_name = table.relation_id().display_name();
        let progress = existing_progress.get(&relation_name)?;
        if !snapshot_progress_complete_at_boundary(progress, &consistent_lsn) {
            return None;
        }
        let progress_copied_rows =
            i64_to_u64_count("snapshot copied_rows", progress.copied_rows).ok()?;
        copied_rows += progress_copied_rows;
        tables.push(SnapshotCopyTableSummary {
            relation: relation_name,
            state: SnapshotRunState::CopyComplete.to_string(),
            copied_rows: progress_copied_rows,
            skipped: true,
            watermark_lsn: progress.watermark_lsn.clone().unwrap_or_default(),
        });
    }

    Some(SnapshotCopySummary {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        run_id: run_id.to_string(),
        state: existing_run.state.to_string(),
        slot: existing_run.slot_name.clone(),
        consistent_lsn,
        selected_table_count: plan.selected_tables.len(),
        table_count: tables.len(),
        skipped_table_count: tables.len(),
        copied_rows,
        handoff_proof_command: snapshot_handoff_proof_command(true, run_id, &tables),
        tables,
        next_commands: vec![
            "trellara relay --config <config>".to_string(),
            "trellara apply --config <config>".to_string(),
            "trellara verify --config <config>".to_string(),
        ],
        consistency_note:
            "snapshot run already reached stream handoff; selected tables were already copied"
                .to_string(),
        handoff_blocker_codes: Vec::new(),
        recovery_actions: Vec::new(),
    })
}
