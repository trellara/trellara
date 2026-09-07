use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::LakeStragglerPolicy;

pub const RAW_CDC_SOURCE_ACK_BOUNDARY: &str =
    "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit";

pub use crate::raw_cdc_metadata_types::{
    LakeRawCdcEpochMetadataPlan, LakeRawCdcEpochPartitionRow, LakeRawCdcEpochQuarantineRow,
    LakeRawCdcEpochRow, LakeRawCdcEpochSourceRow, LakeRawCdcEpochTableRow,
    LakeRawCdcEpochVerificationRow,
};
pub use crate::raw_cdc_row_intent_types::LakeRawCdcRowIntent;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcWriterConfig {
    pub dataset_id: String,
    pub epoch_id: String,
    pub source_bucket_count: u32,
    pub required_sources: BTreeSet<String>,
    pub straggler_policy: LakeStragglerPolicy,
}

impl LakeRawCdcWriterConfig {
    pub fn new(
        dataset_id: impl Into<String>,
        epoch_id: impl Into<String>,
        source_bucket_count: u32,
    ) -> Self {
        Self {
            dataset_id: dataset_id.into(),
            epoch_id: epoch_id.into(),
            source_bucket_count,
            required_sources: BTreeSet::new(),
            straggler_policy: LakeStragglerPolicy::WaitAllRequired,
        }
    }

    pub fn with_required_sources(
        mut self,
        required_sources: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.required_sources = required_sources
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<_>>();
        self
    }

    pub fn with_straggler_policy(mut self, straggler_policy: LakeStragglerPolicy) -> Self {
        self.straggler_policy = straggler_policy;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochWritePlan {
    pub dataset_id: String,
    pub epoch_id: String,
    pub transaction_count: usize,
    pub change_count: usize,
    pub duplicate_transaction_count: usize,
    pub skipped_dataset_transaction_count: usize,
    pub data_file_count: usize,
    pub checksum_rollup: u64,
    pub visibility_rule: String,
    pub duplicate_replay_evidence: LakeRawCdcDuplicateReplayEvidence,
    pub committer_topology: LakeRawCdcCommitterTopology,
    pub commit_steps: Vec<LakeRawCdcCommitStep>,
    pub recovery_scenarios: Vec<LakeRawCdcRecoveryScenario>,
    pub data_files: Vec<LakeRawCdcDataFilePlan>,
    pub row_intents: Vec<LakeRawCdcRowIntent>,
    pub epoch_metadata: LakeRawCdcEpochMetadataPlan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcDuplicateReplayEvidence {
    pub contract: String,
    pub duplicate_transaction_count: usize,
    pub unique_transaction_count: usize,
    pub row_intent_count: usize,
    pub idempotency_key_count: usize,
    pub replay_safe: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcCommitterTopology {
    pub strategy: String,
    pub committer_count: usize,
    pub table_count: usize,
    pub source_bucket_count: usize,
    pub table_committers: Vec<LakeRawCdcTableCommitter>,
    pub source_ack_boundary: String,
    pub catalog_backpressure_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcTableCommitter {
    pub table_name: String,
    pub relation: String,
    pub source_bucket_count: usize,
    pub data_file_count: usize,
    pub commit_policy: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcCommitStep {
    pub order: u32,
    pub phase: String,
    pub action: String,
    pub durability_gate: String,
    pub recovery_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcRecoveryScenario {
    pub code: String,
    pub trigger: String,
    pub replay_policy: String,
    pub operator_evidence: String,
    pub recovery_action: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcDataFilePlan {
    pub table_name: String,
    pub relation: String,
    pub source_bucket: u32,
    pub source_ids: Vec<String>,
    pub transaction_count: usize,
    pub change_count: usize,
    pub min_commit_lsn: String,
    pub max_commit_lsn: String,
    pub checksum_rollup: u64,
    pub idempotency_key_count: usize,
    pub object_key_hint: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcParquetWriteEvidence {
    pub planned_object_key: String,
    pub table_name: String,
    pub relation: String,
    pub source_bucket: u32,
    pub epoch_id: String,
    pub file_uri: String,
    pub file_format: String,
    pub file_size_in_bytes: u64,
    pub content_sha256: String,
    pub object_version: Option<String>,
    pub record_count: u64,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeRawCdcEpochMetadataParquetWriteEvidence {
    pub planned_object_key: String,
    pub table_name: String,
    pub epoch_id: String,
    pub dataset_id: String,
    pub file_uri: String,
    pub file_format: String,
    pub file_size_in_bytes: u64,
    pub content_sha256: String,
    pub object_version: Option<String>,
    pub record_count: u64,
    pub checksum_rollup: u64,
    pub manifest_digest: String,
    pub iceberg_snapshot_id: String,
    pub raw_table_snapshot_ids: BTreeMap<String, i64>,
}
