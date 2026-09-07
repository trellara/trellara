use crate::base::{SimulationConfig, SimulationReport, SimulationStep};

pub(crate) struct BaseReportInput {
    pub(crate) config: SimulationConfig,
    pub(crate) source_acknowledged_lsn: Option<u64>,
    pub(crate) relay_durable_lsn: Option<u64>,
    pub(crate) target_applied_lsn: Option<u64>,
    pub(crate) stream_published_messages: usize,
    pub(crate) stream_acknowledged_messages: usize,
    pub(crate) applied_transactions: usize,
    pub(crate) dedup_transactions: usize,
    pub(crate) skipped_duplicates: usize,
    pub(crate) injected_failure: Option<String>,
    pub(crate) steps: Vec<SimulationStep>,
}

pub(crate) fn build_base_report(input: BaseReportInput) -> SimulationReport {
    let passed = report_passed(&input);

    SimulationReport {
        seed: input.config.seed,
        failure_point: input.config.failure_point,
        transaction_count: input.config.transaction_count,
        passed,
        source_acknowledged_lsn: input.source_acknowledged_lsn,
        relay_durable_lsn: input.relay_durable_lsn,
        target_applied_lsn: input.target_applied_lsn,
        stream_published_messages: input.stream_published_messages,
        stream_acknowledged_messages: input.stream_acknowledged_messages,
        applied_transactions: input.applied_transactions,
        skipped_duplicates: input.skipped_duplicates,
        injected_failure: input.injected_failure,
        steps: input.steps,
    }
}

fn report_passed(input: &BaseReportInput) -> bool {
    input.injected_failure.is_some()
        && input.applied_transactions == input.config.transaction_count
        && input.dedup_transactions == input.config.transaction_count
        && input.target_applied_lsn == input.relay_durable_lsn
        && input.target_applied_lsn == input.source_acknowledged_lsn
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::FailurePoint;

    fn passing_input() -> BaseReportInput {
        BaseReportInput {
            config: SimulationConfig::new(7, FailurePoint::DuplicateDelivery),
            source_acknowledged_lsn: Some(8),
            relay_durable_lsn: Some(8),
            target_applied_lsn: Some(8),
            stream_published_messages: 9,
            stream_acknowledged_messages: 9,
            applied_transactions: 8,
            dedup_transactions: 8,
            skipped_duplicates: 1,
            injected_failure: Some("duplicate".to_string()),
            steps: Vec::new(),
        }
    }

    #[test]
    fn base_report_passes_when_failure_is_recovered_and_boundaries_align() {
        let report = build_base_report(passing_input());

        assert!(report.passed);
        assert_eq!(report.transaction_count, report.applied_transactions);
        assert_eq!(report.target_applied_lsn, report.relay_durable_lsn);
        assert_eq!(report.target_applied_lsn, report.source_acknowledged_lsn);
    }

    #[test]
    fn base_report_fails_without_an_injected_failure() {
        let mut input = passing_input();
        input.injected_failure = None;

        assert!(!build_base_report(input).passed);
    }

    #[test]
    fn base_report_fails_when_target_commit_boundary_lags_relay() {
        let mut input = passing_input();
        input.target_applied_lsn = Some(7);

        assert!(!build_base_report(input).passed);
    }
}
