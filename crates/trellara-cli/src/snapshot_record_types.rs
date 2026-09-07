use serde::Serialize;
use trellara_checkpoint::SnapshotRunState;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SnapshotCopySummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) run_id: String,
    pub(crate) state: String,
    pub(crate) slot: String,
    pub(crate) consistent_lsn: String,
    pub(crate) selected_table_count: usize,
    pub(crate) table_count: usize,
    pub(crate) skipped_table_count: usize,
    pub(crate) copied_rows: u64,
    pub(crate) tables: Vec<SnapshotCopyTableSummary>,
    pub(crate) next_commands: Vec<String>,
    pub(crate) handoff_proof_command: Option<String>,
    pub(crate) consistency_note: String,
    pub(crate) handoff_blocker_codes: Vec<String>,
    pub(crate) recovery_actions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SnapshotCopyTableSummary {
    pub(crate) relation: String,
    pub(crate) state: String,
    pub(crate) copied_rows: u64,
    pub(crate) skipped: bool,
    pub(crate) watermark_lsn: String,
}

pub(crate) struct SnapshotRunDraft<'a> {
    pub(crate) state: SnapshotRunState,
    pub(crate) slot_name: &'a str,
    pub(crate) consistent_lsn: Option<String>,
    pub(crate) current_relation: Option<String>,
    pub(crate) copied_rows: i64,
    pub(crate) failure_reason: Option<String>,
}
