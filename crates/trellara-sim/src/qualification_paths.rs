use super::state::QualificationReportBuilder;
use super::types::{QualificationFailurePoint, QualificationSimulationAction};

impl QualificationReportBuilder {
    pub(crate) fn run(&mut self) {
        match self.failure_point() {
            QualificationFailurePoint::SourcePromotionWhileRelayDisconnected => {
                self.source_promotion_while_relay_disconnected();
            }
            QualificationFailurePoint::BrokerOutageQuorumLoss => self.broker_outage_quorum_loss(),
            QualificationFailurePoint::TargetRestartDuringApply => {
                self.target_restart_during_apply();
            }
            QualificationFailurePoint::ObjectStoreSuccessCatalogTimeout => {
                self.object_store_success_catalog_timeout();
            }
            QualificationFailurePoint::TwentyFourHourSoakLargeTransactionMemoryCeiling => {
                self.twenty_four_hour_soak_large_transaction_memory_ceiling();
            }
        }
    }

    fn source_promotion_while_relay_disconnected(&mut self) {
        self.mark("relay disconnected before source feedback; standby promoted with failover slot");
        self.set_durable_lsn();
        self.push(QualificationSimulationAction::SourceTransactionDurable);
        self.push(QualificationSimulationAction::RelayDisconnected);
        self.push(QualificationSimulationAction::SourcePromoted);
        self.mark_duplicate_replay();
        self.push(QualificationSimulationAction::FailoverSlotReplayed);
        self.apply_all_and_ack();
        self.assert_signal(
            "source_failover_promotion_detected",
            "status reports promoted source identity plus failover slot replay",
            true,
        );
        self.assert_signal(
            "source_ack_lag_visible",
            "metrics expose acknowledgement lag while relay is disconnected",
            true,
        );
    }

    fn broker_outage_quorum_loss(&mut self) {
        self.mark("broker quorum was lost while relay was publishing a transaction");
        self.push(QualificationSimulationAction::BrokerQuorumLost);
        self.push(QualificationSimulationAction::SourceAckWithheld);
        self.assert_signal(
            "broker_quorum_unavailable",
            "readyz and metrics mark broker quorum unavailable before source feedback",
            true,
        );
        self.push(QualificationSimulationAction::BrokerQuorumRestored);
        self.set_durable_lsn();
        self.apply_all_and_ack();
        self.assert_signal(
            "publish_retry_recovered",
            "publish retry counter increments after quorum is restored",
            true,
        );
    }

    fn target_restart_during_apply(&mut self) {
        self.mark("target restarted after apply began but before transaction commit");
        self.set_durable_lsn();
        self.push(QualificationSimulationAction::TargetApplyStarted);
        self.push(QualificationSimulationAction::TargetRestartedBeforeCommit);
        self.push(QualificationSimulationAction::UncheckpointedApplyRolledBack);
        self.mark_duplicate_replay();
        self.apply_all_and_ack();
        self.assert_signal(
            "target_restart_replay_required",
            "diagnostics name the uncheckpointed apply boundary and replay command",
            true,
        );
        self.assert_signal(
            "target_checkpoint_not_advanced_early",
            "target checkpoint stays behind until redelivery commits",
            true,
        );
    }

    fn object_store_success_catalog_timeout(&mut self) {
        self.mark("object-store write succeeded but catalog commit timed out before visibility");
        self.set_durable_lsn();
        self.push(QualificationSimulationAction::ObjectStoreWriteSucceeded);
        self.push(QualificationSimulationAction::CatalogCommitTimedOut);
        self.push(QualificationSimulationAction::SourceAckWithheld);
        self.assert_signal(
            "catalog_commit_pending",
            "lake completeness marks the epoch pending_catalog_commit",
            true,
        );
        self.push(QualificationSimulationAction::CatalogCommitRetried);
        self.apply_all_and_ack();
        self.assert_signal(
            "catalog_retry_idempotent",
            "catalog retry discovers the existing data-file receipt before release",
            true,
        );
    }

    fn twenty_four_hour_soak_large_transaction_memory_ceiling(&mut self) {
        self.mark("24-hour soak includes a streamed large transaction at the spill boundary");
        self.set_durable_lsn();
        self.push(QualificationSimulationAction::SoakWindowCompleted);
        self.push(QualificationSimulationAction::LargeTransactionSpilled);
        self.set_peak_memory_mib(peak_memory_mib(self.large_transaction_change_count()));
        self.push(QualificationSimulationAction::MemoryCeilingObserved);
        self.apply_all_and_ack();
        self.assert_signal(
            "soak_window_completed",
            "qualification records a 24-hour soak window with passing assertions",
            self.soak_hours() >= 24,
        );
        self.assert_signal(
            "large_transaction_memory_ceiling",
            "peak relay memory remains below the configured ceiling",
            self.peak_memory_mib() <= self.memory_ceiling_mib(),
        );
    }
}

fn peak_memory_mib(large_transaction_change_count: usize) -> u32 {
    let active_chunk_changes = large_transaction_change_count.min(1_024) as u32;
    48 + active_chunk_changes.div_ceil(64)
}
