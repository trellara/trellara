use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpochConfig {
    pub epoch_id: String,
    pub dataset_id: String,
    pub required_sources: BTreeSet<String>,
    pub straggler_policy: LakeStragglerPolicy,
}

impl LakeEpochConfig {
    pub fn new(
        epoch_id: impl Into<String>,
        dataset_id: impl Into<String>,
        required_sources: impl IntoIterator<Item = impl Into<String>>,
        straggler_policy: LakeStragglerPolicy,
    ) -> Self {
        Self {
            epoch_id: epoch_id.into(),
            dataset_id: dataset_id.into(),
            required_sources: required_sources
                .into_iter()
                .map(Into::into)
                .collect::<BTreeSet<_>>(),
            straggler_policy,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LakeStragglerPolicy {
    WaitAllRequired,
    PublishWithGaps { grace_ms: u64 },
    QuarantineOnGap,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LakeCompletenessState {
    Open,
    Sealing,
    Complete,
    CompleteWithGaps,
    Quarantined,
    Reseeding,
    FailedRecoverable,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LakeEpochSourceState {
    Complete,
    Lagging,
    Missing,
    Quarantined,
    Reseeding,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpoch {
    pub epoch_id: String,
    pub dataset_id: String,
    pub state: LakeCompletenessState,
    pub straggler_policy: LakeStragglerPolicy,
    pub required_source_count: usize,
    pub complete_source_count: usize,
    pub missing_source_count: usize,
    pub quarantined_source_count: usize,
    pub transaction_count: usize,
    pub change_count: usize,
    pub checksum_rollup: u64,
    pub sources: Vec<LakeEpochSource>,
    pub tables: Vec<LakeEpochTable>,
    pub partitions: Vec<LakeEpochPartition>,
    pub verification: LakeEpochVerification,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpochSource {
    pub source_id: String,
    pub state: LakeEpochSourceState,
    pub start_lsn: Option<String>,
    pub end_lsn: Option<String>,
    pub transaction_count: usize,
    pub change_count: usize,
    pub checksum_rollup: u64,
    pub gap_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpochTable {
    pub relation: String,
    pub transaction_count: usize,
    pub change_count: usize,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpochPartition {
    pub source_id: String,
    pub partition_id: u32,
    pub first_commit_lsn: Option<String>,
    pub last_commit_lsn: Option<String>,
    pub transaction_count: usize,
    pub event_count: usize,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpochVerification {
    pub stream_transaction_count: usize,
    pub stream_change_count: usize,
    pub checksum_rollup: u64,
    pub status: LakeEpochVerificationStatus,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LakeEpochVerificationStatus {
    Match,
    Mismatch,
    Unknown,
}
