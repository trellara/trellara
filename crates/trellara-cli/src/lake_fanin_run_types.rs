use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LakeFaninRunSummary {
    pub(crate) dataset_id: String,
    pub(crate) epoch_id: String,
    pub(crate) mode: String,
    pub(crate) trial_status: LakeFaninRunStatus,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) duplicate_replay_count: usize,
    pub(crate) skipped_dataset_transaction_count: usize,
    pub(crate) data_file_count: usize,
    pub(crate) source_bucket_count: usize,
    pub(crate) replay_safe: bool,
    pub(crate) spark_release_gate: String,
    pub(crate) source_ack_boundary: String,
    pub(crate) catalog_backpressure_rule: String,
    pub(crate) bounded_trial_note: String,
    pub(crate) writer_plan: trellara_lake::LakeRawCdcEpochWritePlan,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LakeFaninRunStatus {
    PlannedDryRun,
    PlannedWithDuplicateReplays,
    BlockedNoDataFiles,
    BlockedEpochNotConsumable,
}
