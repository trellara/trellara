use serde::{Deserialize, Serialize};
use trellara_lake::{LakeCompletenessState, LakeEpochVerificationStatus};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetFanInFailurePoint {
    OfflineStoresPublishWithGaps,
    LateStoreRecoveryCompletesEpoch,
    DuplicateStoreTransactionReplay,
    ConflictingDuplicateQuarantine,
}

impl FleetFanInFailurePoint {
    pub const ALL: [Self; 4] = [
        Self::OfflineStoresPublishWithGaps,
        Self::LateStoreRecoveryCompletesEpoch,
        Self::DuplicateStoreTransactionReplay,
        Self::ConflictingDuplicateQuarantine,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OfflineStoresPublishWithGaps => "fleet_fanin_offline_stores_publish_with_gaps",
            Self::LateStoreRecoveryCompletesEpoch => {
                "fleet_fanin_late_store_recovery_completes_epoch"
            }
            Self::DuplicateStoreTransactionReplay => {
                "fleet_fanin_duplicate_store_transaction_replay"
            }
            Self::ConflictingDuplicateQuarantine => "fleet_fanin_conflicting_duplicate_quarantine",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetFanInSimulationConfig {
    pub seed: u64,
    pub failure_point: FleetFanInFailurePoint,
    pub dataset_id: String,
    pub store_count: usize,
    pub offline_store_count: usize,
    pub duplicate_replay_count: usize,
}

impl FleetFanInSimulationConfig {
    pub fn new(seed: u64, failure_point: FleetFanInFailurePoint) -> Self {
        Self {
            seed,
            failure_point,
            dataset_id: "retail_sales".to_string(),
            store_count: 12,
            offline_store_count: 3,
            duplicate_replay_count: 2,
        }
    }

    pub fn with_dataset_id(mut self, dataset_id: impl Into<String>) -> Self {
        self.dataset_id = dataset_id.into();
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetFanInSimulationReport {
    pub seed: u64,
    pub failure_point: FleetFanInFailurePoint,
    pub epoch_id: String,
    pub dataset_id: String,
    pub required_source_count: usize,
    pub complete_source_count: usize,
    pub missing_source_count: usize,
    pub quarantined_source_count: usize,
    pub transaction_count: usize,
    pub change_count: usize,
    pub duplicate_replay_count: usize,
    pub straggler_policy: String,
    pub initial_state: LakeCompletenessState,
    pub recovered_state: Option<LakeCompletenessState>,
    pub verification_status: LakeEpochVerificationStatus,
    pub passed: bool,
    pub injected_failure: Option<String>,
    pub source_watermarks: Vec<FleetFanInSourceWatermark>,
    pub table_rollups: Vec<FleetFanInTableRollup>,
    pub partition_rollups: Vec<FleetFanInPartitionRollup>,
    pub quarantine_entries: Vec<FleetFanInQuarantineEntry>,
    pub steps: Vec<FleetFanInSimulationStep>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetFanInSourceWatermark {
    pub source_id: String,
    pub state: String,
    pub start_lsn: Option<String>,
    pub end_lsn: Option<String>,
    pub transaction_count: usize,
    pub change_count: usize,
    pub gap_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetFanInTableRollup {
    pub relation: String,
    pub transaction_count: usize,
    pub change_count: usize,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetFanInPartitionRollup {
    pub source_id: String,
    pub partition_id: u32,
    pub first_commit_lsn: Option<String>,
    pub last_commit_lsn: Option<String>,
    pub transaction_count: usize,
    pub event_count: usize,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetFanInQuarantineEntry {
    pub source_id: String,
    pub transaction_id: Option<String>,
    pub commit_lsn: Option<String>,
    pub reason: String,
    pub details: String,
    pub recovery_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FleetFanInSimulationStep {
    pub source_id: Option<String>,
    pub action: FleetFanInSimulationAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetFanInSimulationAction {
    SourceEnvelopePublished,
    SourceEnvelopeReplayedDuplicate,
    SourceMissingAtEpochSeal,
    EpochPublishedWithGaps,
    LateSourceEnvelopeArrived,
    EpochRecomputedComplete,
    ConflictingDuplicateDetected,
    EpochQuarantined,
}
