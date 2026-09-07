use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationFailurePoint {
    SourcePromotionWhileRelayDisconnected,
    BrokerOutageQuorumLoss,
    TargetRestartDuringApply,
    ObjectStoreSuccessCatalogTimeout,
    TwentyFourHourSoakLargeTransactionMemoryCeiling,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QualificationSimulationConfig {
    pub seed: u64,
    pub failure_point: QualificationFailurePoint,
    pub transaction_count: usize,
    pub soak_hours: u32,
    pub large_transaction_change_count: usize,
    pub memory_ceiling_mib: u32,
}

impl QualificationSimulationConfig {
    pub fn new(seed: u64, failure_point: QualificationFailurePoint) -> Self {
        Self {
            seed,
            failure_point,
            transaction_count: 12,
            soak_hours: 24,
            large_transaction_change_count: 50_000,
            memory_ceiling_mib: 128,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QualificationSimulationReport {
    pub seed: u64,
    pub failure_point: QualificationFailurePoint,
    pub durable_boundary: String,
    pub invariant: String,
    pub passed: bool,
    pub transaction_count: usize,
    pub applied_transactions: usize,
    pub duplicate_replays: usize,
    pub source_acknowledged_lsn: Option<u64>,
    pub durable_lsn: Option<u64>,
    pub target_applied_lsn: Option<u64>,
    pub soak_hours: u32,
    pub large_transaction_change_count: usize,
    pub peak_memory_mib: u32,
    pub memory_ceiling_mib: u32,
    pub injected_failure: Option<String>,
    pub recovery_command: String,
    pub observability_assertions: Vec<QualificationObservabilityAssertion>,
    pub steps: Vec<QualificationSimulationStep>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QualificationObservabilityAssertion {
    pub code: String,
    pub passed: bool,
    pub signal: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QualificationSimulationStep {
    pub boundary: String,
    pub action: QualificationSimulationAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationSimulationAction {
    SourceTransactionDurable,
    RelayDisconnected,
    SourcePromoted,
    FailoverSlotReplayed,
    BrokerQuorumLost,
    SourceAckWithheld,
    BrokerQuorumRestored,
    TargetApplyStarted,
    TargetRestartedBeforeCommit,
    UncheckpointedApplyRolledBack,
    ObjectStoreWriteSucceeded,
    CatalogCommitTimedOut,
    CatalogCommitRetried,
    SoakWindowCompleted,
    LargeTransactionSpilled,
    MemoryCeilingObserved,
    AppliedAndCheckpointed,
    SourceAcked,
    ObservabilityAsserted,
}
