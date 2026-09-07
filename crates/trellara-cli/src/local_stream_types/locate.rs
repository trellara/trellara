use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamLocateSummary {
    pub(crate) root: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: Option<String>,
    pub(crate) topics_scanned: Vec<String>,
    pub(crate) matches: Vec<LocalStreamLocateMatch>,
    pub(crate) match_count: usize,
    pub(crate) boundary: LocalStreamLocateBoundarySummary,
    pub(crate) exact_boundary: bool,
    pub(crate) replay_safe: bool,
    pub(crate) replay_warnings: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamLocateMatch {
    pub(crate) topic: String,
    pub(crate) offset: i64,
    pub(crate) next_offset: i64,
    pub(crate) message_kind: String,
    pub(crate) source_id: Option<String>,
    pub(crate) dataset_id: Option<String>,
    pub(crate) commit_lsn: Option<String>,
    pub(crate) partition_id: Option<u32>,
    pub(crate) partition_count: Option<usize>,
    pub(crate) key: String,
    pub(crate) seek_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamLocateBoundarySummary {
    pub(crate) mode: String,
    pub(crate) complete: bool,
    pub(crate) status: String,
    pub(crate) required_message_kinds: Vec<String>,
    pub(crate) found_message_kinds: Vec<String>,
    pub(crate) missing_message_kinds: Vec<String>,
    pub(crate) metadata_conflicts: Vec<String>,
    pub(crate) participating_partition_count: Option<usize>,
    pub(crate) found_partition_ids: Vec<u32>,
    pub(crate) missing_partition_ids: Vec<u32>,
    pub(crate) found_partition_count: usize,
    pub(crate) missing_partition_count: Option<usize>,
}
