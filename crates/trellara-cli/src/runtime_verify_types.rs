use serde::Serialize;

use crate::{ChecksumStatus, TableReseedSummary, TableVerifySummary};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VerifySummary {
    pub(crate) source_watermark_lsn: String,
    pub(crate) target_watermark_lsn: String,
    pub(crate) converged: bool,
    pub(crate) checksum_status: ChecksumStatus,
    pub(crate) tables: Vec<TableVerifySummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ReseedSummary {
    pub(crate) source_watermark_lsn: String,
    pub(crate) table_count: usize,
    pub(crate) copied_rows: u64,
    pub(crate) tables: Vec<TableReseedSummary>,
}
