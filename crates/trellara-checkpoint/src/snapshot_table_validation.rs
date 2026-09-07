use crate::lsn_validation::validate_optional_nonzero_lsn;
use crate::snapshot_table_identity::validate_snapshot_table_progress_identity_unchanged;
use crate::validation_guards::{require_no_surrounding_whitespace, require_non_empty};
use crate::{CheckpointError, Result, SnapshotRunState, SnapshotTableProgress};

pub(crate) fn validate_snapshot_table_progress_record(
    progress: &SnapshotTableProgress,
) -> Result<()> {
    require_non_empty(
        &progress.source_id,
        "snapshot table progress source_id must not be empty",
    )?;
    require_non_empty(
        &progress.dataset_id,
        "snapshot table progress dataset_id must not be empty",
    )?;
    require_non_empty(
        &progress.run_id,
        "snapshot table progress run_id must not be empty",
    )?;
    require_non_empty(
        &progress.relation,
        "snapshot table progress relation must not be empty",
    )?;
    require_no_surrounding_whitespace(
        &progress.source_id,
        "snapshot table progress source_id must not contain surrounding whitespace",
    )?;
    require_no_surrounding_whitespace(
        &progress.dataset_id,
        "snapshot table progress dataset_id must not contain surrounding whitespace",
    )?;
    require_no_surrounding_whitespace(
        &progress.run_id,
        "snapshot table progress run_id must not contain surrounding whitespace",
    )?;
    require_no_surrounding_whitespace(
        &progress.relation,
        "snapshot table progress relation must not contain surrounding whitespace",
    )?;

    if progress.copied_rows < 0 {
        return Err(CheckpointError::Store(format!(
            "snapshot table progress for {} cannot record negative copied rows {}",
            progress.relation, progress.copied_rows
        )));
    }
    if watermark_required(progress.state)
        && progress
            .watermark_lsn
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
    {
        return Err(CheckpointError::Store(format!(
            "snapshot table progress for {} cannot enter {} without a watermark LSN",
            progress.relation, progress.state
        )));
    }
    validate_optional_nonzero_lsn(
        "snapshot table progress",
        "watermark_lsn",
        progress.watermark_lsn.as_deref(),
    )?;

    Ok(())
}

pub(crate) fn validate_snapshot_table_progress_update(
    current: &SnapshotTableProgress,
    next: &SnapshotTableProgress,
) -> Result<()> {
    validate_snapshot_table_progress_identity_unchanged(current, next)?;
    current.state.validate_transition(next.state)?;
    if next.copied_rows < current.copied_rows {
        return Err(CheckpointError::Store(format!(
            "snapshot table progress for {} cannot move copied rows backward from {} to {}",
            current.relation, current.copied_rows, next.copied_rows
        )));
    }

    if let Some(current_watermark) = current.watermark_lsn.as_deref() {
        match next.watermark_lsn.as_deref() {
            Some(next_watermark) if next_watermark == current_watermark => {}
            Some(next_watermark) => {
                return Err(CheckpointError::Store(format!(
                    "snapshot table progress for {} cannot change watermark from {} to {}",
                    current.relation, current_watermark, next_watermark
                )));
            }
            None => {
                return Err(CheckpointError::Store(format!(
                    "snapshot table progress for {} cannot clear watermark {}",
                    current.relation, current_watermark
                )));
            }
        }
    }

    Ok(())
}

fn watermark_required(state: SnapshotRunState) -> bool {
    matches!(
        state,
        SnapshotRunState::CopyComplete
            | SnapshotRunState::StreamHandoffReady
            | SnapshotRunState::Streaming
            | SnapshotRunState::Verified
            | SnapshotRunState::FailedRecoverable
    )
}
