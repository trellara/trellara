use std::collections::HashMap;

use trellara_checkpoint::{self, SnapshotHandoffReadinessInput, SnapshotTableProgress};

pub(crate) fn snapshot_handoff_ready_for_flow(
    source_id: &str,
    dataset_id: &str,
    run_id: &str,
    handoff_relations: &[String],
    progress: &HashMap<String, SnapshotTableProgress>,
    consistent_lsn: &str,
) -> bool {
    let input = readiness_input(
        source_id,
        dataset_id,
        run_id,
        handoff_relations,
        progress,
        consistent_lsn,
    );
    trellara_checkpoint::snapshot_handoff_ready_at_boundary(input).unwrap_or(false)
}

pub(crate) fn snapshot_progress_complete_at_boundary(
    progress: &SnapshotTableProgress,
    consistent_lsn: &str,
) -> bool {
    trellara_checkpoint::snapshot_progress_complete_at_boundary(progress, consistent_lsn)
}

pub(crate) fn snapshot_boundary_copied_rows_for_flow(
    source_id: &str,
    dataset_id: &str,
    run_id: &str,
    handoff_relations: &[String],
    progress: &HashMap<String, SnapshotTableProgress>,
    consistent_lsn: &str,
) -> i64 {
    let input = readiness_input(
        source_id,
        dataset_id,
        run_id,
        handoff_relations,
        progress,
        consistent_lsn,
    );
    trellara_checkpoint::snapshot_boundary_copied_rows(input).unwrap_or(0)
}

pub(crate) fn snapshot_handoff_recovery_for_flow(
    source_id: &str,
    dataset_id: &str,
    run_id: &str,
    handoff_relations: &[String],
    progress: &HashMap<String, SnapshotTableProgress>,
    consistent_lsn: &str,
) -> (Vec<String>, Vec<String>) {
    let input = readiness_input(
        source_id,
        dataset_id,
        run_id,
        handoff_relations,
        progress,
        consistent_lsn,
    );
    match trellara_checkpoint::snapshot_handoff_readiness_report(input) {
        Ok(report) if report.ready => (Vec::new(), Vec::new()),
        Ok(report) => (
            report
                .blockers
                .into_iter()
                .map(|blocker| blocker.code)
                .collect(),
            report.recovery_actions,
        ),
        Err(error) => (
            vec!["invalid_snapshot_handoff_evidence".to_string()],
            vec![format!("repair snapshot handoff evidence: {error}")],
        ),
    }
}

fn readiness_input<'a>(
    source_id: &'a str,
    dataset_id: &'a str,
    run_id: &'a str,
    relations: &'a [String],
    progress: &'a HashMap<String, SnapshotTableProgress>,
    consistent_lsn: &'a str,
) -> SnapshotHandoffReadinessInput<'a> {
    SnapshotHandoffReadinessInput {
        source_id,
        dataset_id,
        run_id,
        consistent_lsn,
        relations,
        progress,
    }
}
