use crate::fleet_fanin::{FleetFanInSimulationAction, FleetFanInSimulationReport};
use crate::fleet_fanin_envelopes::{append_conflicting_duplicate_replay, append_duplicate_replays};
use crate::fleet_fanin_policy::publish_with_gaps_policy;
use crate::fleet_fanin_quarantine::mark_conflicting_duplicate_quarantine;
use crate::fleet_fanin_state::FleetFanInScenario;
use trellara_lake::LakeError;
use trellara_protocol::TransactionEnvelope;

impl FleetFanInScenario {
    pub(crate) fn run_duplicate_store_transaction_replay(
        mut self,
        mut envelopes: Vec<TransactionEnvelope>,
    ) -> FleetFanInSimulationReport {
        append_duplicate_replays(
            &self.workload,
            self.config.duplicate_replay_count,
            &mut envelopes,
            &mut self.steps,
        );
        let initial = self.summary(&envelopes, publish_with_gaps_policy());
        self.push_step(None, FleetFanInSimulationAction::EpochPublishedWithGaps);
        let expected_transactions = self.workload.online_envelopes.len();
        let duplicate_replay_count = self.config.duplicate_replay_count;
        self.report_from_epoch(
            initial.clone(),
            None,
            duplicate_replay_count,
            initial.transaction_count == expected_transactions
                && initial.change_count == expected_transactions,
            None,
        )
    }

    pub(crate) fn run_conflicting_duplicate_quarantine(
        mut self,
        mut envelopes: Vec<TransactionEnvelope>,
    ) -> FleetFanInSimulationReport {
        append_conflicting_duplicate_replay(&self.workload, &mut envelopes, &mut self.steps);
        let error = self
            .summarize_epoch_result(&envelopes, publish_with_gaps_policy())
            .expect_err("conflicting duplicate should quarantine epoch");
        self.push_step(
            None,
            FleetFanInSimulationAction::ConflictingDuplicateDetected,
        );
        self.push_step(None, FleetFanInSimulationAction::EpochQuarantined);
        let quarantined_envelope = self.workload.online_envelopes.first().map(|envelope| {
            (
                envelope.source_id.clone(),
                envelope.transaction_id.clone(),
                envelope.commit_lsn.clone(),
            )
        });
        let initial = self.summary(&self.workload.online_envelopes, publish_with_gaps_policy());
        let mut report = self.report_from_epoch(
            initial,
            None,
            1,
            matches!(error, LakeError::ConflictingDuplicate { .. }),
            Some(error.to_string()),
        );
        mark_conflicting_duplicate_quarantine(&mut report, &error, quarantined_envelope);
        report
    }
}
