use crate::snapshot::{SnapshotSimulationConfig, SnapshotSimulationReport, SnapshotSimulationStep};

pub(crate) struct SnapshotReportInput {
    pub(crate) config: SnapshotSimulationConfig,
    pub(crate) copied_tables: usize,
    pub(crate) stream_replayed_transactions: usize,
    pub(crate) verification_matched: bool,
    pub(crate) handoff_recorded: bool,
    pub(crate) stream_started: bool,
    pub(crate) contract_refreshed: bool,
    pub(crate) injected_failure: Option<String>,
    pub(crate) steps: Vec<SnapshotSimulationStep>,
}

pub(crate) fn build_snapshot_report(input: SnapshotReportInput) -> SnapshotSimulationReport {
    let passed = report_passed(&input);

    SnapshotSimulationReport {
        seed: input.config.seed,
        failure_point: input.config.failure_point,
        table_count: input.config.table_count,
        copied_tables: input.copied_tables,
        writes_after_snapshot: input.config.writes_after_snapshot,
        stream_replayed_transactions: input.stream_replayed_transactions,
        verification_matched: input.verification_matched,
        handoff_recorded: input.handoff_recorded,
        stream_started: input.stream_started,
        contract_refreshed: input.contract_refreshed,
        passed,
        injected_failure: input.injected_failure,
        steps: input.steps,
    }
}

fn report_passed(input: &SnapshotReportInput) -> bool {
    input.injected_failure.is_some()
        && input.handoff_recorded
        && input.stream_started
        && input.verification_matched
        && input.copied_tables == input.config.table_count
        && input.stream_replayed_transactions == input.config.writes_after_snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::SnapshotFailurePoint;

    fn passing_input() -> SnapshotReportInput {
        SnapshotReportInput {
            config: SnapshotSimulationConfig::new(19, SnapshotFailurePoint::DdlDuringTableCopy),
            copied_tables: 4,
            stream_replayed_transactions: 6,
            verification_matched: true,
            handoff_recorded: true,
            stream_started: true,
            contract_refreshed: true,
            injected_failure: Some("schema drift".to_string()),
            steps: Vec::new(),
        }
    }

    #[test]
    fn snapshot_report_passes_when_handoff_stream_and_verification_complete() {
        let report = build_snapshot_report(passing_input());

        assert!(report.passed);
        assert_eq!(report.table_count, report.copied_tables);
        assert_eq!(
            report.writes_after_snapshot,
            report.stream_replayed_transactions
        );
    }

    #[test]
    fn snapshot_report_fails_when_stream_has_not_started_after_handoff() {
        let mut input = passing_input();
        input.stream_started = false;

        assert!(!build_snapshot_report(input).passed);
    }

    #[test]
    fn snapshot_report_fails_when_verification_does_not_match() {
        let mut input = passing_input();
        input.verification_matched = false;

        assert!(!build_snapshot_report(input).passed);
    }
}
