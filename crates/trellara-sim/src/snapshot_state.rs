use crate::failure_injection::FailureInjection;
use crate::snapshot::{
    SnapshotFailurePoint, SnapshotSimulationAction, SnapshotSimulationConfig,
    SnapshotSimulationReport,
};
use crate::snapshot_report::{build_snapshot_report, SnapshotReportInput};
use crate::snapshot_steps::SnapshotSimulationSteps;
use crate::snapshot_table_copy::copy_snapshot_table_with_retry;
use crate::snapshot_table_tracker::SnapshotTableTracker;

pub(crate) struct SnapshotSimState {
    config: SnapshotSimulationConfig,
    tables: Vec<String>,
    copied_tables: SnapshotTableTracker,
    stream_replayed_transactions: usize,
    verification_matched: bool,
    handoff_recorded: bool,
    stream_started: bool,
    contract_refreshed: bool,
    failure: FailureInjection,
    steps: SnapshotSimulationSteps,
}

impl SnapshotSimState {
    pub(crate) fn new(config: SnapshotSimulationConfig, tables: Vec<String>) -> Self {
        Self {
            config,
            tables,
            copied_tables: SnapshotTableTracker::default(),
            stream_replayed_transactions: 0,
            verification_matched: false,
            handoff_recorded: false,
            stream_started: false,
            contract_refreshed: false,
            failure: FailureInjection::default(),
            steps: SnapshotSimulationSteps::default(),
        }
    }

    pub(crate) fn run(&mut self) {
        self.push_step(None, SnapshotSimulationAction::SlotCreated);
        self.push_step(None, SnapshotSimulationAction::SnapshotExported);

        for table in self.tables.clone() {
            self.contract_refreshed |= copy_snapshot_table_with_retry(
                table,
                self.config.failure_point,
                &mut self.failure,
                &mut self.steps,
                &mut self.copied_tables,
            );
        }

        self.handoff_recorded = true;
        self.push_step(None, SnapshotSimulationAction::HandoffRecorded);

        if self.should_inject(SnapshotFailurePoint::HandoffRecordedBeforeStreamStart) {
            self.mark_failure(
                "relay crashed after snapshot handoff was recorded before stream start",
            );
            self.push_step(
                None,
                SnapshotSimulationAction::RelayCrashedBeforeStreamStart,
            );
        }

        self.stream_started = true;
        self.push_step(None, SnapshotSimulationAction::StreamStarted);

        for _ in 0..self.config.writes_after_snapshot {
            self.stream_replayed_transactions += 1;
            self.push_step(None, SnapshotSimulationAction::PostSnapshotWriteReplayed);
        }

        self.verification_matched = self.copied_tables.is_complete(self.config.table_count)
            && self.stream_replayed_transactions == self.config.writes_after_snapshot;
        self.push_step(None, SnapshotSimulationAction::VerifiedConverged);
    }

    fn should_inject(&mut self, failure_point: SnapshotFailurePoint) -> bool {
        self.failure
            .should_inject(self.config.failure_point, failure_point)
    }

    fn mark_failure(&mut self, failure: impl Into<String>) {
        self.failure.mark(failure);
    }

    fn push_step(&mut self, relation: Option<String>, action: SnapshotSimulationAction) {
        self.steps.push(relation, action);
    }

    pub(crate) fn report(self) -> SnapshotSimulationReport {
        build_snapshot_report(SnapshotReportInput {
            config: self.config,
            copied_tables: self.copied_tables.copied_count(),
            stream_replayed_transactions: self.stream_replayed_transactions,
            verification_matched: self.verification_matched,
            handoff_recorded: self.handoff_recorded,
            stream_started: self.stream_started,
            contract_refreshed: self.contract_refreshed,
            injected_failure: self.failure.into_description(),
            steps: self.steps.into_steps(),
        })
    }
}
