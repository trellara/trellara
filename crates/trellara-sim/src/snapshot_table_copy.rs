use crate::failure_injection::FailureInjection;
use crate::snapshot::{SnapshotFailurePoint, SnapshotSimulationAction};
use crate::snapshot_steps::SnapshotSimulationSteps;
use crate::snapshot_table_copy_failures::maybe_retry_recoverable_table_copy_failure;
use crate::snapshot_table_tracker::SnapshotTableTracker;

pub(crate) fn copy_snapshot_table_with_retry(
    table: String,
    configured_failure_point: SnapshotFailurePoint,
    failure: &mut FailureInjection,
    steps: &mut SnapshotSimulationSteps,
    copied_tables: &mut SnapshotTableTracker,
) -> bool {
    steps.push(
        Some(table.clone()),
        SnapshotSimulationAction::TableCopyStarted,
    );

    let contract_refreshed = maybe_retry_recoverable_table_copy_failure(
        &table,
        configured_failure_point,
        failure,
        steps,
    );

    copied_tables.mark_copied(table.clone());
    steps.push(
        Some(table.clone()),
        SnapshotSimulationAction::TableCopyCompleted,
    );
    maybe_skip_duplicate_copy_attempt(&table, configured_failure_point, failure, steps);
    contract_refreshed
}

fn maybe_skip_duplicate_copy_attempt(
    table: &str,
    configured_failure_point: SnapshotFailurePoint,
    failure: &mut FailureInjection,
    steps: &mut SnapshotSimulationSteps,
) {
    if !failure.should_inject(
        configured_failure_point,
        SnapshotFailurePoint::DuplicateCopyAttempt,
    ) {
        return;
    }

    failure.mark("operator retried a completed snapshot table copy");
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::DuplicateTableCopyAttempted,
    );
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::DuplicateTableCopySkipped,
    );
}
