use crate::snapshot_handoff_readiness_types::SnapshotHandoffReadinessBlocker;
use crate::{SnapshotRunState, SnapshotTableProgress};

pub(crate) fn add_missing_progress_blocker(
    blockers: &mut Vec<SnapshotHandoffReadinessBlocker>,
    actions: &mut Vec<String>,
    relation: &str,
) {
    add_blocker(
        blockers,
        actions,
        "missing_table_progress",
        relation,
        "no persisted table-copy progress exists for the selected relation",
        "rerun or resume the snapshot until every selected table records copy_complete",
    );
}

pub(crate) fn add_progress_blocker(
    blockers: &mut Vec<SnapshotHandoffReadinessBlocker>,
    actions: &mut Vec<String>,
    relation: &str,
    progress: &SnapshotTableProgress,
    consistent_lsn: &str,
) {
    if progress.state != SnapshotRunState::CopyComplete {
        add_blocker(
            blockers,
            actions,
            "table_not_copy_complete",
            relation,
            "table copy has not reached copy_complete",
            "resume table copy before starting CDC catch-up",
        );
    } else if progress.watermark_lsn.is_none() {
        add_blocker(
            blockers,
            actions,
            "missing_watermark_lsn",
            relation,
            "table copy completed without a durable handoff watermark",
            "record the table-copy completion at the exported consistent_lsn",
        );
    } else {
        add_blocker(
            blockers,
            actions,
            "watermark_lsn_mismatch",
            relation,
            &format!(
                "table watermark_lsn {:?} does not match consistent_lsn {consistent_lsn:?}",
                progress.watermark_lsn
            ),
            "recreate the slot or run a fresh snapshot handoff at one consistent_lsn",
        );
    }
}

fn add_blocker(
    blockers: &mut Vec<SnapshotHandoffReadinessBlocker>,
    actions: &mut Vec<String>,
    code: &str,
    relation: &str,
    detail: &str,
    action: &str,
) {
    blockers.push(SnapshotHandoffReadinessBlocker {
        code: code.to_string(),
        relation: relation.to_string(),
        detail: detail.to_string(),
    });
    push_unique_action(actions, action);
}

fn push_unique_action(actions: &mut Vec<String>, action: &str) {
    if !actions.iter().any(|existing| existing == action) {
        actions.push(action.to_string());
    }
}
