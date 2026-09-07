use serde::Serialize;

use super::locate::LocalStreamLocateBoundarySummary;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamReconstructSummary {
    pub(crate) root: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: Option<String>,
    pub(crate) boundary: LocalStreamLocateBoundarySummary,
    pub(crate) manifest_offset: i64,
    pub(crate) commit_offset: i64,
    pub(crate) partition_offsets: Vec<LocalStreamReconstructPartitionOffset>,
    pub(crate) partition_chunk_count: usize,
    pub(crate) reconstructed_change_count: usize,
    pub(crate) source_order: Vec<u32>,
    pub(crate) proof: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamReconstructPartitionOffset {
    pub(crate) partition_id: u32,
    pub(crate) offset: i64,
}
