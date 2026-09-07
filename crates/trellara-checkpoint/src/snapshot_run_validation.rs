use crate::lsn_validation::validate_optional_nonzero_lsn;
use crate::snapshot_run_identity::validate_snapshot_run_identity_unchanged;
use crate::validation_guards::{require_no_surrounding_whitespace, require_non_empty};
use crate::{CheckpointError, Result, SnapshotRun, SnapshotRunState};

pub(crate) fn validate_snapshot_run_record(run: &SnapshotRun) -> Result<()> {
    require_non_empty(&run.source_id, "snapshot run source_id must not be empty")?;
    require_non_empty(&run.dataset_id, "snapshot run dataset_id must not be empty")?;
    require_non_empty(&run.run_id, "snapshot run run_id must not be empty")?;
    require_no_surrounding_whitespace(
        &run.source_id,
        "snapshot run source_id must not contain surrounding whitespace",
    )?;
    require_no_surrounding_whitespace(
        &run.dataset_id,
        "snapshot run dataset_id must not contain surrounding whitespace",
    )?;
    require_no_surrounding_whitespace(
        &run.run_id,
        "snapshot run run_id must not contain surrounding whitespace",
    )?;

    if run.copied_rows < 0 {
        return Err(CheckpointError::Store(format!(
            "snapshot run {} cannot record negative copied rows {}",
            run.run_id, run.copied_rows
        )));
    }

    validate_failure_reason(run)?;

    if slot_name_required(run.state) && run.slot_name.trim().is_empty() {
        return Err(CheckpointError::Store(format!(
            "snapshot run {} cannot enter {} without a slot name",
            run.run_id, run.state
        )));
    }

    if consistent_lsn_required(run.state)
        && run
            .consistent_lsn
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
    {
        return Err(CheckpointError::Store(format!(
            "snapshot run {} cannot enter {} without a consistent LSN",
            run.run_id, run.state
        )));
    }
    validate_optional_nonzero_lsn(
        "snapshot run",
        "consistent_lsn",
        run.consistent_lsn.as_deref(),
    )?;

    Ok(())
}

fn validate_failure_reason(run: &SnapshotRun) -> Result<()> {
    let Some(reason) = run.failure_reason.as_deref() else {
        if run.state == SnapshotRunState::FailedRecoverable {
            return Err(CheckpointError::Store(format!(
                "snapshot run {} cannot enter failed_recoverable without a failure reason",
                run.run_id
            )));
        }
        return Ok(());
    };

    require_non_empty(
        reason,
        format!(
            "snapshot run {} failure_reason must not be empty",
            run.run_id
        ),
    )?;
    require_no_surrounding_whitespace(
        reason,
        format!(
            "snapshot run {} failure_reason must not contain surrounding whitespace",
            run.run_id
        ),
    )
}

pub(crate) fn validate_snapshot_run_update(
    current: &SnapshotRun,
    next: &SnapshotRun,
) -> Result<()> {
    validate_snapshot_run_identity_unchanged(current, next)?;
    current.state.validate_transition(next.state)?;
    if next.copied_rows < current.copied_rows {
        return Err(CheckpointError::Store(format!(
            "snapshot run {} cannot move copied rows backward from {} to {}",
            current.run_id, current.copied_rows, next.copied_rows
        )));
    }

    if let Some(current_lsn) = current.consistent_lsn.as_deref() {
        match next.consistent_lsn.as_deref() {
            Some(next_lsn) if next_lsn == current_lsn => {}
            Some(next_lsn) => {
                return Err(CheckpointError::Store(format!(
                    "snapshot run {} cannot change consistent LSN from {} to {}",
                    current.run_id, current_lsn, next_lsn
                )));
            }
            None => {
                return Err(CheckpointError::Store(format!(
                    "snapshot run {} cannot clear consistent LSN {}",
                    current.run_id, current_lsn
                )));
            }
        }
    }

    Ok(())
}

fn slot_name_required(state: SnapshotRunState) -> bool {
    matches!(
        state,
        SnapshotRunState::SlotCreated
            | SnapshotRunState::SnapshotExported
            | SnapshotRunState::CopyingTable
            | SnapshotRunState::CopyComplete
            | SnapshotRunState::StreamHandoffReady
            | SnapshotRunState::Streaming
            | SnapshotRunState::Verified
            | SnapshotRunState::FailedRecoverable
    )
}

fn consistent_lsn_required(state: SnapshotRunState) -> bool {
    matches!(
        state,
        SnapshotRunState::SnapshotExported
            | SnapshotRunState::CopyingTable
            | SnapshotRunState::CopyComplete
            | SnapshotRunState::StreamHandoffReady
            | SnapshotRunState::Streaming
            | SnapshotRunState::Verified
            | SnapshotRunState::FailedRecoverable
    )
}
