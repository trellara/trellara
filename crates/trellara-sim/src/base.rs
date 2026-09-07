use serde::{Deserialize, Serialize};

use crate::base_state::SimState;
use crate::transaction::generated_transactions;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailurePoint {
    PublishAckLoss,
    CrashAfterPublishBeforeSourceAck,
    SourceFailoverAfterPublishBeforeAck,
    DuplicateDelivery,
    TargetFailureBeforeCommit,
    TargetQuarantineRepairReplay,
    CheckpointFailureDuringApply,
    StreamAckLossAfterApply,
}

impl FailurePoint {
    pub const ALL: [Self; 8] = [
        Self::PublishAckLoss,
        Self::CrashAfterPublishBeforeSourceAck,
        Self::SourceFailoverAfterPublishBeforeAck,
        Self::DuplicateDelivery,
        Self::TargetFailureBeforeCommit,
        Self::TargetQuarantineRepairReplay,
        Self::CheckpointFailureDuringApply,
        Self::StreamAckLossAfterApply,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PublishAckLoss => "publish_ack_loss",
            Self::CrashAfterPublishBeforeSourceAck => "crash_after_publish_before_source_ack",
            Self::SourceFailoverAfterPublishBeforeAck => "source_failover_after_publish_before_ack",
            Self::DuplicateDelivery => "duplicate_delivery",
            Self::TargetFailureBeforeCommit => "target_failure_before_commit",
            Self::TargetQuarantineRepairReplay => "target_quarantine_repair_replay",
            Self::CheckpointFailureDuringApply => "checkpoint_failure_during_apply",
            Self::StreamAckLossAfterApply => "stream_ack_loss_after_apply",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub seed: u64,
    pub failure_point: FailurePoint,
    pub transaction_count: usize,
}

impl SimulationConfig {
    pub fn new(seed: u64, failure_point: FailurePoint) -> Self {
        Self {
            seed,
            failure_point,
            transaction_count: 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationReport {
    pub seed: u64,
    pub failure_point: FailurePoint,
    pub transaction_count: usize,
    pub passed: bool,
    pub source_acknowledged_lsn: Option<u64>,
    pub relay_durable_lsn: Option<u64>,
    pub target_applied_lsn: Option<u64>,
    pub stream_published_messages: usize,
    pub stream_acknowledged_messages: usize,
    pub applied_transactions: usize,
    pub skipped_duplicates: usize,
    pub injected_failure: Option<String>,
    pub steps: Vec<SimulationStep>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationStep {
    pub transaction_id: String,
    pub commit_lsn: u64,
    pub action: SimulationAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationAction {
    Published,
    PublishAckLost,
    RelayDurableCheckpointed,
    RelayCrashedBeforeSourceAck,
    SourceFailoverBeforeFeedback,
    SourceAcked,
    Delivered,
    DuplicateDelivered,
    TargetFailureBeforeCommit,
    TargetQuarantined,
    OperatorMarkedReplayReady,
    CheckpointFailureRolledBackApply,
    AppliedAndCheckpointed,
    SkippedDuplicate,
    StreamAckLost,
    StreamAcked,
}

pub fn run_simulation(config: SimulationConfig) -> SimulationReport {
    let transactions = generated_transactions(config.seed, config.transaction_count);
    let mut state = SimState::new(config, transactions);
    state.run();
    state.report()
}

pub fn run_default_suite(seed: u64) -> Vec<SimulationReport> {
    FailurePoint::ALL
        .into_iter()
        .enumerate()
        .map(|(index, failure_point)| {
            run_simulation(SimulationConfig::new(
                seed.wrapping_add(index as u64),
                failure_point,
            ))
        })
        .collect()
}
