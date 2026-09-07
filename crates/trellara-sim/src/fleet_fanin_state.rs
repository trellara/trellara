use crate::fleet_fanin::{
    FleetFanInFailurePoint, FleetFanInSimulationAction, FleetFanInSimulationConfig,
    FleetFanInSimulationReport,
};
use crate::fleet_fanin_envelopes::{append_late_source_envelopes, record_initial_epoch_inputs};
use crate::fleet_fanin_epoch::{build_fleet_epoch_config, summarize_fleet_epoch};
use crate::fleet_fanin_policy::publish_with_gaps_policy;
use crate::fleet_fanin_report::{build_fleet_fanin_report, FleetFanInReportInput};
use crate::fleet_fanin_steps::FleetFanInSteps;
use crate::fleet_fanin_workload::FleetFanInWorkload;
use trellara_lake::{
    LakeCompletenessState, LakeEpoch, LakeEpochConfig, LakeError, LakeStragglerPolicy,
};
use trellara_protocol::TransactionEnvelope;

pub(crate) struct FleetFanInScenario {
    pub(crate) config: FleetFanInSimulationConfig,
    pub(crate) workload: FleetFanInWorkload,
    pub(crate) steps: FleetFanInSteps,
}

impl FleetFanInScenario {
    pub(crate) fn new(config: FleetFanInSimulationConfig) -> Self {
        let workload = FleetFanInWorkload::new(&config);
        Self {
            config,
            workload,
            steps: FleetFanInSteps::default(),
        }
    }

    pub(crate) fn run(mut self) -> FleetFanInSimulationReport {
        let envelopes = record_initial_epoch_inputs(&self.workload, &mut self.steps);

        match self.config.failure_point {
            FleetFanInFailurePoint::OfflineStoresPublishWithGaps => {
                self.run_offline_stores_publish_with_gaps(envelopes)
            }
            FleetFanInFailurePoint::LateStoreRecoveryCompletesEpoch => {
                self.run_late_store_recovery(envelopes)
            }
            FleetFanInFailurePoint::DuplicateStoreTransactionReplay => {
                self.run_duplicate_store_transaction_replay(envelopes)
            }
            FleetFanInFailurePoint::ConflictingDuplicateQuarantine => {
                self.run_conflicting_duplicate_quarantine(envelopes)
            }
        }
    }

    fn run_offline_stores_publish_with_gaps(
        mut self,
        envelopes: Vec<TransactionEnvelope>,
    ) -> FleetFanInSimulationReport {
        let initial = self.summary(&envelopes, publish_with_gaps_policy());
        self.push_step(None, FleetFanInSimulationAction::EpochPublishedWithGaps);
        self.report_from_epoch(initial, None, envelopes.len(), true, None)
    }

    fn run_late_store_recovery(
        mut self,
        mut envelopes: Vec<TransactionEnvelope>,
    ) -> FleetFanInSimulationReport {
        let initial = self.summary(&envelopes, publish_with_gaps_policy());
        self.push_step(None, FleetFanInSimulationAction::EpochPublishedWithGaps);
        append_late_source_envelopes(&self.workload, &mut envelopes, &mut self.steps);
        let recovered = self.summary(&envelopes, LakeStragglerPolicy::WaitAllRequired);
        self.push_step(None, FleetFanInSimulationAction::EpochRecomputedComplete);
        let mut report = self.report_from_epoch(
            recovered.clone(),
            Some(initial.state),
            envelopes.len(),
            recovered.state == LakeCompletenessState::Complete,
            None,
        );
        report.initial_state = initial.state;
        report.recovered_state = Some(recovered.state);
        report
    }

    pub(crate) fn summary(
        &self,
        envelopes: &[TransactionEnvelope],
        straggler_policy: LakeStragglerPolicy,
    ) -> LakeEpoch {
        summarize_fleet_epoch(&self.epoch_config(straggler_policy), envelopes)
            .expect("fleet fan-in epoch summary")
    }

    pub(crate) fn summarize_epoch_result(
        &self,
        envelopes: &[TransactionEnvelope],
        straggler_policy: LakeStragglerPolicy,
    ) -> Result<LakeEpoch, LakeError> {
        summarize_fleet_epoch(&self.epoch_config(straggler_policy), envelopes)
    }

    fn epoch_config(&self, straggler_policy: LakeStragglerPolicy) -> LakeEpochConfig {
        build_fleet_epoch_config(
            self.config.seed,
            &self.config.dataset_id,
            self.workload.sources.iter().cloned(),
            straggler_policy,
        )
    }

    pub(crate) fn report_from_epoch(
        self,
        epoch: LakeEpoch,
        recovered_from: Option<LakeCompletenessState>,
        duplicate_replay_count: usize,
        passed: bool,
        injected_failure: Option<String>,
    ) -> FleetFanInSimulationReport {
        build_fleet_fanin_report(FleetFanInReportInput {
            seed: self.config.seed,
            failure_point: self.config.failure_point,
            epoch,
            recovered_from,
            duplicate_replay_count,
            passed,
            injected_failure,
            quarantine_entries: Vec::new(),
            steps: self.steps.into_steps(),
        })
    }

    pub(crate) fn push_step(
        &mut self,
        source_id: Option<String>,
        action: FleetFanInSimulationAction,
    ) {
        self.steps.push(source_id, action);
    }
}
