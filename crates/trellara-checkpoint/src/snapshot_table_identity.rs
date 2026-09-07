use crate::{CheckpointError, Result, SnapshotTableProgress};

pub(crate) fn validate_snapshot_table_progress_identity_unchanged(
    current: &SnapshotTableProgress,
    next: &SnapshotTableProgress,
) -> Result<()> {
    if current.source_id != next.source_id {
        return Err(snapshot_table_progress_identity_error(
            current,
            "source_id",
            &current.source_id,
            &next.source_id,
        ));
    }
    if current.dataset_id != next.dataset_id {
        return Err(snapshot_table_progress_identity_error(
            current,
            "dataset_id",
            &current.dataset_id,
            &next.dataset_id,
        ));
    }
    if current.run_id != next.run_id {
        return Err(snapshot_table_progress_identity_error(
            current,
            "run_id",
            &current.run_id,
            &next.run_id,
        ));
    }
    if current.relation != next.relation {
        return Err(snapshot_table_progress_identity_error(
            current,
            "relation",
            &current.relation,
            &next.relation,
        ));
    }
    Ok(())
}

fn snapshot_table_progress_identity_error(
    current: &SnapshotTableProgress,
    field: &'static str,
    current_value: &str,
    next_value: &str,
) -> CheckpointError {
    CheckpointError::Store(format!(
        "snapshot table progress for {} cannot change {field} from {current_value} to {next_value}",
        current.relation
    ))
}
