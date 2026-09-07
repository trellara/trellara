use crate::SnapshotCopySummary;

pub(crate) fn snapshot_handoff_evidence(snapshot: &SnapshotCopySummary) -> String {
    format!(
        "snapshot run {} is {} at consistent_lsn={} with {}/{} tables copied; selected_table_count={}; skipped_tables={}; table_states={}; handoff_proof_command={}; snapshot_handoff_blocker_codes={}; snapshot_handoff_recovery_actions={}; next={}; note={}",
        snapshot.run_id,
        snapshot.state,
        snapshot.consistent_lsn,
        copied_table_count(snapshot),
        snapshot.table_count,
        snapshot.selected_table_count,
        snapshot.skipped_table_count,
        snapshot_table_states(snapshot),
        snapshot_handoff_command(snapshot),
        list_or_none(&snapshot.handoff_blocker_codes),
        list_or_none(&snapshot.recovery_actions),
        snapshot_next_command(snapshot),
        snapshot.consistency_note
    )
}

fn copied_table_count(snapshot: &SnapshotCopySummary) -> usize {
    snapshot
        .table_count
        .saturating_sub(snapshot.skipped_table_count)
}

fn snapshot_table_states(snapshot: &SnapshotCopySummary) -> String {
    if snapshot.tables.is_empty() {
        return "none".to_string();
    }
    snapshot
        .tables
        .iter()
        .map(|table| {
            format!(
                "{}:{} rows={} watermark_lsn={} skipped={}",
                table.relation, table.state, table.copied_rows, table.watermark_lsn, table.skipped
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn snapshot_handoff_command(snapshot: &SnapshotCopySummary) -> String {
    snapshot
        .handoff_proof_command
        .clone()
        .unwrap_or_else(|| "none".to_string())
}

fn snapshot_next_command(snapshot: &SnapshotCopySummary) -> String {
    snapshot
        .next_commands
        .first()
        .cloned()
        .unwrap_or_else(|| "trellara verify --config <config>".to_string())
}

fn list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join("|")
    }
}
