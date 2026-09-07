use crate::lsn_validation::validate_required_nonzero_lsn;
use crate::snapshot_handoff_blockers::{add_missing_progress_blocker, add_progress_blocker};
use crate::snapshot_handoff_readiness_types::{
    SnapshotHandoffReadinessInput, SnapshotHandoffReadinessReport,
};
use crate::validation_guards::require_non_empty;
use crate::{CheckpointError, Result, SnapshotRunState, SnapshotTableProgress};

pub fn snapshot_handoff_ready_at_boundary(
    input: SnapshotHandoffReadinessInput<'_>,
) -> Result<bool> {
    Ok(snapshot_handoff_readiness_report(input)?.ready)
}

pub fn snapshot_boundary_copied_rows(input: SnapshotHandoffReadinessInput<'_>) -> Result<i64> {
    Ok(snapshot_handoff_readiness_report(input)?.copied_rows_at_boundary)
}

pub fn snapshot_handoff_readiness_report(
    input: SnapshotHandoffReadinessInput<'_>,
) -> Result<SnapshotHandoffReadinessReport> {
    validate_input(&input)?;
    let mut copied_rows_at_boundary = 0_i64;
    let mut blockers = Vec::new();
    let mut recovery_actions = Vec::new();

    for relation in input.relations {
        match input.progress.get(relation) {
            Some(progress) => {
                validate_progress_identity(&input, relation, progress)?;
                if snapshot_progress_complete_at_boundary(progress, input.consistent_lsn) {
                    copied_rows_at_boundary += progress.copied_rows;
                } else {
                    add_progress_blocker(
                        &mut blockers,
                        &mut recovery_actions,
                        relation,
                        progress,
                        input.consistent_lsn,
                    );
                }
            }
            None => add_missing_progress_blocker(&mut blockers, &mut recovery_actions, relation),
        }
    }

    Ok(SnapshotHandoffReadinessReport {
        ready: blockers.is_empty(),
        relation_count: input.relations.len(),
        copied_rows_at_boundary,
        blockers,
        recovery_actions,
    })
}

pub fn snapshot_progress_complete_at_boundary(
    progress: &SnapshotTableProgress,
    consistent_lsn: &str,
) -> bool {
    progress.state == SnapshotRunState::CopyComplete
        && progress.watermark_lsn.as_deref() == Some(consistent_lsn)
}

fn validate_input(input: &SnapshotHandoffReadinessInput<'_>) -> Result<()> {
    require_non_empty(
        input.source_id,
        "snapshot handoff source_id must not be empty",
    )?;
    require_non_empty(
        input.dataset_id,
        "snapshot handoff dataset_id must not be empty",
    )?;
    require_non_empty(input.run_id, "snapshot handoff run_id must not be empty")?;
    require_non_empty(
        input.consistent_lsn,
        "snapshot handoff consistent_lsn must not be empty",
    )?;
    validate_required_nonzero_lsn("snapshot handoff", "consistent_lsn", input.consistent_lsn)?;
    if input.relations.is_empty() {
        return Err(CheckpointError::Store(
            "snapshot handoff must include at least one relation".to_string(),
        ));
    }
    Ok(())
}

fn validate_progress_identity(
    input: &SnapshotHandoffReadinessInput<'_>,
    relation: &str,
    progress: &SnapshotTableProgress,
) -> Result<()> {
    validate_field("source_id", input.source_id, &progress.source_id, relation)?;
    validate_field(
        "dataset_id",
        input.dataset_id,
        &progress.dataset_id,
        relation,
    )?;
    validate_field("run_id", input.run_id, &progress.run_id, relation)?;
    validate_field("relation", relation, &progress.relation, relation)
}

fn validate_field(field: &'static str, expected: &str, actual: &str, relation: &str) -> Result<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(CheckpointError::Store(format!(
            "snapshot handoff progress for {relation} has {field} {actual:?}, expected {expected:?}"
        )))
    }
}
