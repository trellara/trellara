use crate::failure_injection::FailureInjection;
use crate::snapshot::{SnapshotFailurePoint, SnapshotSimulationAction};
use crate::snapshot_steps::SnapshotSimulationSteps;

pub(crate) fn maybe_retry_recoverable_table_copy_failure(
    table: &str,
    configured_failure_point: SnapshotFailurePoint,
    failure: &mut FailureInjection,
    steps: &mut SnapshotSimulationSteps,
) -> bool {
    if should_retry_after_source_crash(configured_failure_point, failure, table, steps) {
        return false;
    }
    if should_retry_after_relay_crash(configured_failure_point, failure, table, steps) {
        return false;
    }
    if should_retry_after_target_crash(configured_failure_point, failure, table, steps) {
        return false;
    }
    should_retry_after_schema_drift(configured_failure_point, failure, table, steps)
}

fn should_retry_after_source_crash(
    configured_failure_point: SnapshotFailurePoint,
    failure: &mut FailureInjection,
    table: &str,
    steps: &mut SnapshotSimulationSteps,
) -> bool {
    if !failure.should_inject(
        configured_failure_point,
        SnapshotFailurePoint::SourceCrashDuringTableCopy,
    ) {
        return false;
    }

    failure.mark("source connection failed while exporting a snapshot table");
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::SourceCrashedDuringTableCopy,
    );
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::SnapshotMarkedFailedRecoverable,
    );
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::TableCopyRetried,
    );
    true
}

fn should_retry_after_relay_crash(
    configured_failure_point: SnapshotFailurePoint,
    failure: &mut FailureInjection,
    table: &str,
    steps: &mut SnapshotSimulationSteps,
) -> bool {
    if !failure.should_inject(
        configured_failure_point,
        SnapshotFailurePoint::RelayCrashDuringTableCopy,
    ) {
        return false;
    }

    failure.mark("relay crashed while copying a snapshot table");
    push_crash_retry(table, steps);
    true
}

fn should_retry_after_target_crash(
    configured_failure_point: SnapshotFailurePoint,
    failure: &mut FailureInjection,
    table: &str,
    steps: &mut SnapshotSimulationSteps,
) -> bool {
    if !failure.should_inject(
        configured_failure_point,
        SnapshotFailurePoint::TargetCrashDuringTableCopy,
    ) {
        return false;
    }

    failure.mark("target connection failed while copying a snapshot table");
    push_crash_retry(table, steps);
    true
}

fn should_retry_after_schema_drift(
    configured_failure_point: SnapshotFailurePoint,
    failure: &mut FailureInjection,
    table: &str,
    steps: &mut SnapshotSimulationSteps,
) -> bool {
    if !failure.should_inject(
        configured_failure_point,
        SnapshotFailurePoint::DdlDuringTableCopy,
    ) {
        return false;
    }

    failure.mark(
        "source schema changed during table copy; snapshot handoff was withheld until contract refresh",
    );
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::SchemaDriftDetected,
    );
    steps.push(None, SnapshotSimulationAction::HandoffWithheld);
    steps.push(None, SnapshotSimulationAction::ContractRefreshed);
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::TableCopyRetried,
    );
    true
}

fn push_crash_retry(table: &str, steps: &mut SnapshotSimulationSteps) {
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::TableCopyCrashed,
    );
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::SnapshotMarkedFailedRecoverable,
    );
    steps.push(
        Some(table.to_string()),
        SnapshotSimulationAction::TableCopyRetried,
    );
}
