use crate::{CheckpointError, Result, SnapshotRun};

pub(crate) fn validate_snapshot_run_identity_unchanged(
    current: &SnapshotRun,
    next: &SnapshotRun,
) -> Result<()> {
    if current.source_id != next.source_id {
        return Err(snapshot_run_identity_error(
            current,
            "source_id",
            &current.source_id,
            &next.source_id,
        ));
    }
    if current.dataset_id != next.dataset_id {
        return Err(snapshot_run_identity_error(
            current,
            "dataset_id",
            &current.dataset_id,
            &next.dataset_id,
        ));
    }
    if current.run_id != next.run_id {
        return Err(snapshot_run_identity_error(
            current,
            "run_id",
            &current.run_id,
            &next.run_id,
        ));
    }
    Ok(())
}

fn snapshot_run_identity_error(
    current: &SnapshotRun,
    field: &'static str,
    current_value: &str,
    next_value: &str,
) -> CheckpointError {
    CheckpointError::Store(format!(
        "snapshot run {} cannot change {field} from {current_value} to {next_value}",
        current.run_id
    ))
}
