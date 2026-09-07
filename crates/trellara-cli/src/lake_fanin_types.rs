use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeFaninVerifySummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) contract: String,
    pub(crate) stream_epoch_path: String,
    pub(crate) lake_epoch_path: String,
    pub(crate) epoch_id: String,
    pub(crate) status: LakeFaninVerifyStatus,
    pub(crate) stream_state: trellara_lake::LakeCompletenessState,
    pub(crate) lake_state: trellara_lake::LakeCompletenessState,
    pub(crate) stream_required_source_count: usize,
    pub(crate) stream_complete_source_count: usize,
    pub(crate) stream_missing_source_count: usize,
    pub(crate) stream_quarantined_source_count: usize,
    pub(crate) lake_required_source_count: usize,
    pub(crate) lake_complete_source_count: usize,
    pub(crate) lake_missing_source_count: usize,
    pub(crate) lake_quarantined_source_count: usize,
    pub(crate) source_counts_match: bool,
    pub(crate) stream_checksum_rollup: u64,
    pub(crate) lake_checksum_rollup: u64,
    pub(crate) checksum_rollup_match: bool,
    pub(crate) stream_customer_decision: String,
    pub(crate) lake_customer_decision: String,
    pub(crate) matched_check_count: usize,
    pub(crate) mismatch_count: usize,
    pub(crate) warning_mismatch_count: usize,
    pub(crate) blocker_mismatch_count: usize,
    pub(crate) mismatches: Vec<LakeFaninVerifyMismatch>,
    pub(crate) spark_consumption_allowed: bool,
    pub(crate) spark_consumption_contract: &'static str,
    pub(crate) spark_consumption_gate: String,
    pub(crate) proof_command: String,
    pub(crate) recommended_next_steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeFaninCompletenessSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) contract: String,
    pub(crate) positioning: String,
    pub(crate) epoch_id: String,
    pub(crate) completeness_state: trellara_lake::LakeCompletenessState,
    pub(crate) decision: LakeFaninCompletenessDecision,
    pub(crate) accepted_complete_with_gaps: bool,
    pub(crate) spark_consumption_allowed: bool,
    pub(crate) spark_consumption_contract: &'static str,
    pub(crate) spark_consumption_gate: String,
    pub(crate) required_source_count: usize,
    pub(crate) complete_source_count: usize,
    pub(crate) missing_source_count: usize,
    pub(crate) quarantined_source_count: usize,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
    pub(crate) manifest_digest: String,
    pub(crate) source_state_counts: LakeFaninCompletenessSourceStateCounts,
    pub(crate) source_rows: Vec<crate::LakeEpochSourceWatermark>,
    pub(crate) table_rollups: Vec<crate::LakeEpochTableRollup>,
    pub(crate) quarantine_entries: Vec<crate::LakeEpochQuarantineEntry>,
    pub(crate) raw_cdc_tables: Vec<LakeFaninCompletenessTableRef>,
    pub(crate) epoch_metadata_tables: Vec<LakeFaninCompletenessTableRef>,
    pub(crate) spark_templates: Vec<LakeFaninCompletenessSparkTemplate>,
    pub(crate) proof_artifacts: Vec<String>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) recovery_path: Vec<String>,
    pub(crate) deferred_sink_work: Vec<String>,
    pub(crate) verification: LakeFaninVerifySummary,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LakeFaninCompletenessDecision {
    Ready,
    ReadyWithAcceptedGaps,
    BlockedNeedsGapAcceptance,
    BlockedVerificationMismatch,
    BlockedNonConsumableEpoch,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct LakeFaninCompletenessSourceStateCounts {
    pub(crate) complete: usize,
    pub(crate) lagging: usize,
    pub(crate) missing: usize,
    pub(crate) quarantined: usize,
    pub(crate) reseeding: usize,
    pub(crate) unknown: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeFaninCompletenessTableRef {
    pub(crate) materialization: String,
    pub(crate) table_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeFaninCompletenessSparkTemplate {
    pub(crate) kind: String,
    pub(crate) artifact: String,
    pub(crate) command: String,
    pub(crate) gate: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeFaninVerifyMismatch {
    pub(crate) field: String,
    pub(crate) stream_value: String,
    pub(crate) lake_value: String,
    pub(crate) severity: LakeFaninVerifyMismatchSeverity,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LakeFaninVerifyStatus {
    Match,
    Mismatch,
    Blocked,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LakeFaninVerifyMismatchSeverity {
    Warning,
    Blocker,
}
