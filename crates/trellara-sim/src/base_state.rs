use std::collections::VecDeque;

use crate::base::{FailurePoint, SimulationAction, SimulationConfig, SimulationReport};
use crate::base_apply::apply_next_message;
use crate::base_relay::{inject_relay_failure, RelayFailure};
use crate::base_report::{build_base_report, BaseReportInput};
use crate::base_steps::BaseSimulationSteps;
use crate::base_stream::BaseStream;
use crate::base_target::BaseTarget;
use crate::failure_injection::FailureInjection;
use crate::transaction::Transaction;

pub(crate) struct SimState {
    config: SimulationConfig,
    source: VecDeque<Transaction>,
    stream: BaseStream,
    relay_durable_lsn: Option<u64>,
    source_acknowledged_lsn: Option<u64>,
    target_applied_lsn: Option<u64>,
    target: BaseTarget,
    failure: FailureInjection,
    steps: BaseSimulationSteps,
}

impl SimState {
    pub(crate) fn new(config: SimulationConfig, transactions: Vec<Transaction>) -> Self {
        Self {
            config,
            source: VecDeque::from(transactions),
            stream: BaseStream::default(),
            relay_durable_lsn: None,
            source_acknowledged_lsn: None,
            target_applied_lsn: None,
            target: BaseTarget::default(),
            failure: FailureInjection::default(),
            steps: BaseSimulationSteps::default(),
        }
    }

    pub(crate) fn run(&mut self) {
        while let Some(transaction) = self.source.pop_front() {
            self.relay_transaction(transaction);
        }

        while !self.stream.is_empty() {
            self.apply_next_message();
        }
    }

    fn relay_transaction(&mut self, transaction: Transaction) {
        self.publish(transaction.clone());

        match inject_relay_failure(
            &mut self.failure,
            self.config.failure_point,
            &transaction,
            &mut self.steps,
        ) {
            RelayFailure::RecoverableDuplicate => {
                self.publish(transaction.clone());
            }
            RelayFailure::StopBeforeDurableCheckpoint => {
                self.publish(transaction.clone());
            }
            RelayFailure::None => {}
        }

        self.relay_durable_lsn = Some(transaction.lsn);
        self.push_step(&transaction, SimulationAction::RelayDurableCheckpointed);

        if self.should_inject(FailurePoint::CrashAfterPublishBeforeSourceAck) {
            self.mark_failure("relay crashed after durable checkpoint before source feedback");
            self.push_step(&transaction, SimulationAction::RelayCrashedBeforeSourceAck);
            self.publish(transaction.clone());
        }

        if self.should_inject(FailurePoint::SourceFailoverAfterPublishBeforeAck) {
            self.mark_failure(
                "source primary failed after durable publish before old-source feedback; relay resumed on promoted source failover slot",
            );
            self.push_step(&transaction, SimulationAction::SourceFailoverBeforeFeedback);
            self.publish(transaction.clone());
        }

        self.source_acknowledged_lsn = Some(transaction.lsn);
        self.push_step(&transaction, SimulationAction::SourceAcked);
    }

    fn publish(&mut self, transaction: Transaction) {
        self.stream.publish(transaction.clone());
        self.push_step(&transaction, SimulationAction::Published);
    }

    fn apply_next_message(&mut self) {
        if let Some(applied_lsn) = apply_next_message(
            &mut self.stream,
            &mut self.target,
            &mut self.failure,
            self.config.failure_point,
            &mut self.steps,
        ) {
            self.target_applied_lsn = Some(applied_lsn);
        }
    }

    fn should_inject(&mut self, failure_point: FailurePoint) -> bool {
        self.failure
            .should_inject(self.config.failure_point, failure_point)
    }

    fn mark_failure(&mut self, failure: impl Into<String>) {
        self.failure.mark(failure);
    }

    fn push_step(&mut self, transaction: &Transaction, action: SimulationAction) {
        self.steps
            .push(transaction.id.clone(), transaction.lsn, action);
    }

    pub(crate) fn report(self) -> SimulationReport {
        build_base_report(BaseReportInput {
            config: self.config,
            source_acknowledged_lsn: self.source_acknowledged_lsn,
            relay_durable_lsn: self.relay_durable_lsn,
            target_applied_lsn: self.target_applied_lsn,
            stream_published_messages: self.stream.published_messages(),
            stream_acknowledged_messages: self.stream.acknowledged_messages(),
            applied_transactions: self.target.applied_count(),
            dedup_transactions: self.target.dedup_count(),
            skipped_duplicates: self.target.skipped_duplicate_count(),
            injected_failure: self.failure.into_description(),
            steps: self.steps.into_steps(),
        })
    }
}
