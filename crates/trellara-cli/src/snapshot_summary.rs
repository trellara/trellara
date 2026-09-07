use trellara_checkpoint::SnapshotRunState;

use crate::{SnapshotCopySummary, SnapshotCopyTableSummary, TrellaraConfig};

pub(crate) struct SnapshotCopySummaryDraft {
    pub(crate) run_id: String,
    pub(crate) slot: String,
    pub(crate) consistent_lsn: String,
    pub(crate) handoff_ready: bool,
    pub(crate) table_count: usize,
    pub(crate) skipped_table_count: usize,
    pub(crate) copied_rows: u64,
    pub(crate) tables: Vec<SnapshotCopyTableSummary>,
    pub(crate) exported_snapshot_name: Option<String>,
    pub(crate) handoff_blocker_codes: Vec<String>,
    pub(crate) recovery_actions: Vec<String>,
}

pub(crate) fn snapshot_copy_summary(
    config: &TrellaraConfig,
    draft: SnapshotCopySummaryDraft,
) -> SnapshotCopySummary {
    let handoff_proof_command =
        snapshot_handoff_proof_command(draft.handoff_ready, &draft.run_id, &draft.tables);
    SnapshotCopySummary {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        run_id: draft.run_id,
        state: if draft.handoff_ready {
            SnapshotRunState::StreamHandoffReady
        } else {
            SnapshotRunState::CopyingTable
        }
        .to_string(),
        slot: draft.slot,
        consistent_lsn: draft.consistent_lsn,
        selected_table_count: draft.table_count,
        table_count: draft.table_count,
        skipped_table_count: draft.skipped_table_count,
        copied_rows: draft.copied_rows,
        tables: draft.tables,
        next_commands: snapshot_next_commands(draft.handoff_ready),
        handoff_proof_command,
        consistency_note: snapshot_consistency_note(
            draft.handoff_ready,
            draft.exported_snapshot_name.is_some(),
        ),
        handoff_blocker_codes: draft.handoff_blocker_codes,
        recovery_actions: draft.recovery_actions,
    }
}

fn snapshot_next_commands(handoff_ready: bool) -> Vec<String> {
    if handoff_ready {
        vec![
            "trellara relay --config <config>".to_string(),
            "trellara apply --config <config>".to_string(),
            "trellara verify --config <config>".to_string(),
        ]
    } else {
        vec!["trellara snapshot --config <config>".to_string()]
    }
}

pub(crate) fn snapshot_handoff_proof_command(
    handoff_ready: bool,
    run_id: &str,
    tables: &[SnapshotCopyTableSummary],
) -> Option<String> {
    if !handoff_ready {
        return None;
    }

    let mut command = format!("trellara snapshot --config <config> --run-id {run_id}");
    if tables.len() == 1 {
        command.push_str(&format!(" --table {}", tables[0].relation));
    }
    Some(command)
}

fn snapshot_consistency_note(handoff_ready: bool, exported_snapshot: bool) -> String {
    if !handoff_ready {
        return "copied the selected tables but withheld stream handoff until every configured relation is copy_complete at the same consistent LSN".to_string();
    }

    if exported_snapshot {
        "copies source tables inside the exported logical snapshot held by the pgoutput replication connection, then hands off at the slot consistent LSN".to_string()
    } else {
        "records the logical slot LSN and uses the existing table copy path with durable resume state"
            .to_string()
    }
}
