use serde::Serialize;
use trellara_checkpoint::ApplyQuarantine;

use crate::LocalStreamLocateBoundarySummary;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuarantineListSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) count: usize,
    pub(crate) records: Vec<ApplyQuarantine>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuarantineClearSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) cleared: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuarantineReplayReadySummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) quarantine_reason: String,
    pub(crate) quarantine_detail: String,
    pub(crate) dedup_removed: bool,
    pub(crate) quarantine_cleared: bool,
    pub(crate) redelivery_required: bool,
    pub(crate) redelivery_topics: Vec<String>,
    pub(crate) redelivery_boundary: Option<LocalStreamLocateBoundarySummary>,
    pub(crate) redelivery_warnings: Vec<String>,
    pub(crate) exact_seek_commands: Vec<String>,
    pub(crate) redelivery_commands: Vec<String>,
    pub(crate) redelivery_hint: String,
    pub(crate) safety_contract: QuarantineReplaySafetyContract,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuarantineReplaySafetyContract {
    pub(crate) exact_boundary_required: bool,
    pub(crate) cleared_state: String,
    pub(crate) redelivery_gate: String,
    pub(crate) next_operator_action: String,
    pub(crate) boundary_evidence: QuarantineReplayBoundaryEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct QuarantineReplayBoundaryEvidence {
    pub(crate) exact_seek_available: bool,
    pub(crate) boundary_status: String,
    pub(crate) boundary_mode: Option<String>,
    pub(crate) complete: Option<bool>,
    pub(crate) exact_seek_command_count: usize,
    pub(crate) required_message_kinds: Vec<String>,
    pub(crate) missing_message_kinds: Vec<String>,
    pub(crate) found_partition_ids: Vec<u32>,
    pub(crate) missing_partition_ids: Vec<u32>,
    pub(crate) metadata_conflicts: Vec<String>,
}
