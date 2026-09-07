use serde::{Deserialize, Serialize};

use crate::LakeEpochScenario;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LakeEpochSourceWatermark {
    pub(crate) source_id: String,
    pub(crate) state: String,
    pub(crate) start_lsn: Option<String>,
    pub(crate) end_lsn: Option<String>,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) gap_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LakeEpochWatermarkRollup {
    pub(crate) global_low_watermark_lsn: Option<String>,
    pub(crate) max_source_watermark_lsn: Option<String>,
    pub(crate) complete_source_count: usize,
    pub(crate) lagging_source_count: usize,
    pub(crate) missing_source_count: usize,
    pub(crate) quarantined_source_count: usize,
    pub(crate) invalid_lsn_source_count: usize,
    pub(crate) invalid_lsn_sources: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LakeEpochTableRollup {
    pub(crate) relation: String,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LakeEpochPartitionRollup {
    pub(crate) source_id: String,
    pub(crate) partition_id: u32,
    pub(crate) first_commit_lsn: Option<String>,
    pub(crate) last_commit_lsn: Option<String>,
    pub(crate) transaction_count: usize,
    pub(crate) event_count: usize,
    pub(crate) checksum_rollup: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LakeEpochPartitionSkew {
    pub(crate) participating_partition_count: usize,
    pub(crate) total_event_count: usize,
    pub(crate) min_event_count: usize,
    pub(crate) max_event_count: usize,
    pub(crate) skew_ratio_basis_points: Option<u64>,
    pub(crate) hottest_partition_ids: Vec<u32>,
    pub(crate) coolest_partition_ids: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LakeEpochQuarantineEntry {
    pub(crate) source_id: String,
    pub(crate) transaction_id: Option<String>,
    pub(crate) commit_lsn: Option<String>,
    pub(crate) reason: String,
    pub(crate) details: String,
    pub(crate) recovery_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LakeEpochSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) contract: String,
    pub(crate) fanin_mode: String,
    pub(crate) scenario: LakeEpochScenario,
    pub(crate) epoch_id: String,
    pub(crate) state: trellara_lake::LakeCompletenessState,
    pub(crate) recovered_state: Option<trellara_lake::LakeCompletenessState>,
    pub(crate) required_source_count: usize,
    pub(crate) complete_source_count: usize,
    pub(crate) missing_source_count: usize,
    pub(crate) quarantined_source_count: usize,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
    pub(crate) duplicate_replay_count: usize,
    pub(crate) straggler_policy: String,
    pub(crate) straggler_policy_decision: String,
    pub(crate) manifest_digest: String,
    pub(crate) watermark_rollup: LakeEpochWatermarkRollup,
    pub(crate) source_watermarks: Vec<LakeEpochSourceWatermark>,
    pub(crate) table_rollups: Vec<LakeEpochTableRollup>,
    pub(crate) partition_rollups: Vec<LakeEpochPartitionRollup>,
    #[serde(default)]
    pub(crate) partition_skew: LakeEpochPartitionSkew,
    pub(crate) quarantine_entries: Vec<LakeEpochQuarantineEntry>,
    pub(crate) verification_status: trellara_lake::LakeEpochVerificationStatus,
    pub(crate) passed: bool,
    pub(crate) injected_failure: Option<String>,
    pub(crate) visibility_boundary: String,
    pub(crate) customer_decision: String,
    pub(crate) proof_command: String,
    pub(crate) recommended_next_steps: Vec<String>,
}
