use super::types::{
    QualificationFailurePoint, QualificationObservabilityAssertion, QualificationSimulationAction,
    QualificationSimulationConfig, QualificationSimulationReport, QualificationSimulationStep,
};

pub(crate) struct QualificationReportBuilder {
    config: QualificationSimulationConfig,
    applied_transactions: usize,
    duplicate_replays: usize,
    source_acknowledged_lsn: Option<u64>,
    durable_lsn: Option<u64>,
    target_applied_lsn: Option<u64>,
    peak_memory_mib: u32,
    injected_failure: Option<String>,
    observability_assertions: Vec<QualificationObservabilityAssertion>,
    steps: Vec<QualificationSimulationStep>,
}

impl QualificationReportBuilder {
    pub(crate) fn new(config: QualificationSimulationConfig) -> Self {
        Self {
            config,
            applied_transactions: 0,
            duplicate_replays: 0,
            source_acknowledged_lsn: None,
            durable_lsn: None,
            target_applied_lsn: None,
            peak_memory_mib: 0,
            injected_failure: None,
            observability_assertions: Vec::new(),
            steps: Vec::new(),
        }
    }

    pub(crate) fn finish(self) -> QualificationSimulationReport {
        let passed = self.passed();
        QualificationSimulationReport {
            seed: self.config.seed,
            failure_point: self.config.failure_point,
            durable_boundary: self.config.failure_point.boundary().to_string(),
            invariant: self.config.failure_point.invariant().to_string(),
            passed,
            transaction_count: self.config.transaction_count,
            applied_transactions: self.applied_transactions,
            duplicate_replays: self.duplicate_replays,
            source_acknowledged_lsn: self.source_acknowledged_lsn,
            durable_lsn: self.durable_lsn,
            target_applied_lsn: self.target_applied_lsn,
            soak_hours: self.config.soak_hours,
            large_transaction_change_count: self.config.large_transaction_change_count,
            peak_memory_mib: self.peak_memory_mib,
            memory_ceiling_mib: self.config.memory_ceiling_mib,
            injected_failure: self.injected_failure,
            recovery_command: self.config.failure_point.recovery_command().to_string(),
            observability_assertions: self.observability_assertions,
            steps: self.steps,
        }
    }

    pub(crate) fn failure_point(&self) -> QualificationFailurePoint {
        self.config.failure_point
    }

    pub(crate) fn large_transaction_change_count(&self) -> usize {
        self.config.large_transaction_change_count
    }

    pub(crate) fn soak_hours(&self) -> u32 {
        self.config.soak_hours
    }

    pub(crate) fn memory_ceiling_mib(&self) -> u32 {
        self.config.memory_ceiling_mib
    }

    pub(crate) fn peak_memory_mib(&self) -> u32 {
        self.peak_memory_mib
    }

    pub(crate) fn set_peak_memory_mib(&mut self, peak_memory_mib: u32) {
        self.peak_memory_mib = peak_memory_mib;
    }

    pub(crate) fn set_durable_lsn(&mut self) {
        self.durable_lsn = Some(final_lsn(self.config.seed, self.config.transaction_count));
    }

    pub(crate) fn mark_duplicate_replay(&mut self) {
        self.duplicate_replays += 1;
    }

    pub(crate) fn apply_all_and_ack(&mut self) {
        self.applied_transactions = self.config.transaction_count;
        self.target_applied_lsn = self.durable_lsn;
        self.push(QualificationSimulationAction::AppliedAndCheckpointed);
        self.source_acknowledged_lsn = self.durable_lsn;
        self.push(QualificationSimulationAction::SourceAcked);
    }

    pub(crate) fn mark(&mut self, failure: impl Into<String>) {
        self.injected_failure = Some(failure.into());
    }

    pub(crate) fn assert_signal(&mut self, code: &str, signal: &str, passed: bool) {
        self.observability_assertions
            .push(QualificationObservabilityAssertion {
                code: code.to_string(),
                passed,
                signal: signal.to_string(),
            });
        self.push(QualificationSimulationAction::ObservabilityAsserted);
    }

    pub(crate) fn push(&mut self, action: QualificationSimulationAction) {
        self.steps.push(QualificationSimulationStep {
            boundary: self.config.failure_point.boundary().to_string(),
            action,
        });
    }

    fn passed(&self) -> bool {
        self.injected_failure.is_some()
            && self.applied_transactions == self.config.transaction_count
            && self.source_acknowledged_lsn == self.durable_lsn
            && self.target_applied_lsn == self.durable_lsn
            && self
                .observability_assertions
                .iter()
                .all(|assertion| assertion.passed)
            && self.soak_load_passed()
    }

    fn soak_load_passed(&self) -> bool {
        self.config.failure_point
            != QualificationFailurePoint::TwentyFourHourSoakLargeTransactionMemoryCeiling
            || (self.config.soak_hours >= 24
                && self.peak_memory_mib <= self.config.memory_ceiling_mib)
    }
}

fn final_lsn(seed: u64, transaction_count: usize) -> u64 {
    16_000 + seed % 1_024 + transaction_count as u64
}
