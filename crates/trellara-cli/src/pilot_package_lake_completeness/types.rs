use serde::Serialize;

use crate::LakeEpochPartitionSkew;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeCompletenessEvidence {
    pub(crate) contract: String,
    pub(crate) epoch_id: String,
    pub(crate) dataset_id: String,
    pub(crate) completeness_state: String,
    pub(crate) verification_status: String,
    pub(crate) spark_consumption_allowed: bool,
    pub(crate) spark_consumption_contract: &'static str,
    pub(crate) spark_consumption_gate: String,
    pub(crate) source_watermark_count: usize,
    pub(crate) table_rollup_count: usize,
    pub(crate) partition_rollup_count: usize,
    pub(crate) partition_skew: LakeEpochPartitionSkew,
    pub(crate) source_counts_match: bool,
    pub(crate) stream_required_source_count: usize,
    pub(crate) stream_complete_source_count: usize,
    pub(crate) stream_missing_source_count: usize,
    pub(crate) stream_quarantined_source_count: usize,
    pub(crate) lake_required_source_count: usize,
    pub(crate) lake_complete_source_count: usize,
    pub(crate) lake_missing_source_count: usize,
    pub(crate) lake_quarantined_source_count: usize,
    pub(crate) required_source_count: usize,
    pub(crate) complete_source_count: usize,
    pub(crate) missing_source_count: usize,
    pub(crate) quarantined_source_count: usize,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup_match: bool,
    pub(crate) stream_checksum_rollup: u64,
    pub(crate) lake_checksum_rollup: u64,
    pub(crate) checksum_rollup: u64,
    pub(crate) source_rows: Vec<LakeCompletenessSourceEvidence>,
    pub(crate) committer_strategy: String,
    pub(crate) source_ack_boundary: String,
    pub(crate) catalog_backpressure_rule: String,
    pub(crate) recovery_scenario_codes: Vec<String>,
    pub(crate) recovery_guidance: trellara_lake::LakeEpochRecoveryGuidance,
    pub(crate) spark_templates: Vec<LakeCompletenessSparkTemplate>,
    pub(crate) review_artifacts: Vec<String>,
    pub(crate) decision_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeCompletenessSourceEvidence {
    pub(crate) source_id: String,
    pub(crate) state: String,
    pub(crate) start_lsn: String,
    pub(crate) end_lsn: String,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
    pub(crate) lag_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeCompletenessSparkTemplate {
    pub(crate) kind: String,
    pub(crate) artifact: String,
    pub(crate) gate: String,
}
